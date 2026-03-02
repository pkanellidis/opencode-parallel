use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::server::OpenCodeServer;
use crate::utils::truncate_str;

const ORCHESTRATOR_SYSTEM_PROMPT: &str = r#"You are an AI task orchestrator. Your job is to analyze user requests and decide how to split them into parallel tasks.

When the user sends a request, respond ONLY with a JSON object in this exact format (no markdown, no code blocks, just raw JSON):
{
  "tasks": [
    {"id": 1, "description": "Brief task description", "prompt": "The exact user request or a portion of it"}
  ],
  "reasoning": "Brief explanation of why you split the tasks this way"
}

Rules:
- If the task is simple and doesn't benefit from parallelization, return a single task with the EXACT user request as the prompt
- If the task can be split into independent subtasks, create multiple tasks
- Each task should be self-contained and not depend on other tasks' outputs
- Use as many tasks as needed to fully parallelize the work
- Keep descriptions under 50 characters
- IMPORTANT: The "prompt" field should contain the user's original request or a subset of it verbatim. Do NOT rewrite, rephrase, or add instructions. Do NOT add any information about yourself or any AI model.

Examples of when to split:
- "Create a web app with auth and database" -> Split into frontend, backend, auth, database tasks
- "Write tests for modules A, B, and C" -> One task per module

Examples of single task (use EXACT user prompt):
- User: "Explain how async/await works" -> prompt: "Explain how async/await works"
- User: "Fix the bug in login.js" -> prompt: "Fix the bug in login.js"
- User: "What model are you?" -> prompt: "What model are you?"

IMPORTANT: Respond ONLY with valid JSON, no other text, no markdown code blocks."#;

const ORCHESTRATOR_ANALYZE_PROMPT: &str = r#"You are an AI task orchestrator. You have dispatched workers to complete tasks, and now you have received their results.

Analyze the worker results and decide if any follow-up tasks are needed.

Respond ONLY with a JSON object in this exact format (no markdown, no code blocks, just raw JSON):
{
  "tasks": [
    {"id": 1, "description": "Brief task description", "prompt": "Detailed prompt for the follow-up task"}
  ],
  "reasoning": "Explanation of why these follow-up tasks are needed, or why no follow-up is needed",
  "complete": true/false
}

Rules:
- Set "complete" to true if the original user request has been fully addressed and no follow-up is needed
- Set "complete" to false if follow-up tasks are required
- If complete is true, return an empty tasks array
- Follow-up tasks should address:
  - Errors or failures in worker results that need to be fixed
  - Integration work needed after parallel tasks complete (e.g., combining components)
  - Additional steps discovered during task execution
  - Quality improvements or refinements needed
- Keep task descriptions under 50 characters
- Each follow-up task should be self-contained
- Do NOT create follow-up tasks for work that was already completed successfully
- Do NOT create unnecessary follow-up tasks - only create them when genuinely needed

Examples of when to create follow-up tasks:
- A worker reported an error that needs fixing
- Multiple workers created separate components that need integration
- A worker discovered additional requirements during execution
- Tests failed and fixes are needed

Examples of when NOT to create follow-up tasks (set complete: true):
- All workers completed successfully with no issues
- The task was informational (explanation, documentation)
- No errors or integration needs were identified

IMPORTANT: Respond ONLY with valid JSON, no other text, no markdown code blocks."#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub reasoning: String,
    #[serde(default)]
    pub complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    pub worker_id: u32,
    pub description: String,
    pub success: bool,
    pub output: String,
}

pub struct Orchestrator {
    server: OpenCodeServer,
    session_id: Option<String>,
    logs: Vec<String>,
    model: Option<String>,
}

impl Orchestrator {
    pub fn new(server: OpenCodeServer) -> Self {
        Self {
            server,
            session_id: None,
            logs: Vec::new(),
            model: None,
        }
    }

    pub fn set_model(&mut self, model: Option<String>) {
        self.model = model;
    }

    fn log(&mut self, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.logs.push(format!("[{}] {}", timestamp, message));
    }

    pub fn get_logs(&self) -> &[String] {
        &self.logs
    }

    pub async fn init(&mut self) -> Result<()> {
        self.log("Initializing orchestrator session...".to_string());
        match self.server.create_session(Some("Orchestrator")).await {
            Ok(session) => {
                self.log(format!(
                    "Session created: {}",
                    &session.id[..8.min(session.id.len())]
                ));
                self.session_id = Some(session.id);
                Ok(())
            }
            Err(e) => {
                self.log(format!("Failed to create session: {}", e));
                Err(e)
            }
        }
    }

    pub fn set_session_id(&mut self, session_id: String) {
        self.log(format!(
            "Using existing session: {}",
            &session_id[..8.min(session_id.len())]
        ));
        self.session_id = Some(session_id);
    }

    pub fn get_session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }

    pub async fn report_worker_results(&mut self, results: &str) -> Result<()> {
        let session_id = self
            .session_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Orchestrator not initialized"))?;

        self.log("Reporting worker results to orchestrator...".to_string());

        let report = format!(
            "WORKER RESULTS (for context in future requests):\n{}",
            results
        );

        let _ = self.server.send_message(&session_id, &report).await?;
        self.log("Worker results reported successfully".to_string());
        Ok(())
    }

    pub async fn analyze_results(
        &mut self,
        original_request: &str,
        worker_results: &[WorkerResult],
    ) -> Result<TaskPlan> {
        self.log("Analyzing worker results for follow-up tasks...".to_string());

        let session_id = self
            .session_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Orchestrator not initialized"))?;

        let results_summary: String = worker_results
            .iter()
            .map(|r| {
                format!(
                    "Worker #{} ({}): {}\nOutput:\n{}",
                    r.worker_id,
                    r.description,
                    if r.success { "SUCCESS" } else { "FAILED" },
                    truncate_str(&r.output, 500)
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let prompt = format!(
            "{}\n\nOriginal user request: {}\n\nWorker results:\n{}",
            ORCHESTRATOR_ANALYZE_PROMPT, original_request, results_summary
        );

        self.log("Sending analysis request to orchestrator AI...".to_string());
        if let Some(ref m) = self.model {
            self.log(format!("Using model: {}", m));
        }

        let response = self
            .server
            .send_message_with_model(&session_id, &prompt, self.model.as_deref())
            .await?;

        let mut full_text = String::new();
        for part in response.parts {
            if let Some(text) = part.text {
                full_text.push_str(&text);
            }
        }

        self.log(format!(
            "Received analysis response ({} chars)",
            full_text.len()
        ));
        self.log(format!("Raw response: {}", truncate_str(&full_text, 200)));

        match self.parse_analysis_response(&full_text) {
            Ok(plan) => {
                self.log(format!(
                    "Analysis complete: {} follow-up task(s), complete={}",
                    plan.tasks.len(),
                    plan.complete
                ));
                for task in &plan.tasks {
                    self.log(format!("  Follow-up #{}: {}", task.id, task.description));
                }
                Ok(plan)
            }
            Err(e) => {
                self.log(format!("Parse error: {}", e));
                Ok(TaskPlan {
                    tasks: vec![],
                    reasoning: "Could not parse analysis response, assuming complete".to_string(),
                    complete: true,
                })
            }
        }
    }

    fn parse_analysis_response(&mut self, response: &str) -> Result<TaskPlan, String> {
        let cleaned = response.trim();

        self.log("Attempt 1: Direct JSON parse".to_string());
        if let Ok(plan) = serde_json::from_str::<TaskPlan>(cleaned) {
            self.log("Success: Direct parse worked".to_string());
            return Ok(plan);
        }

        self.log("Attempt 2: Extract from markdown code blocks".to_string());
        if let Some(json_str) = self.extract_json_from_markdown(cleaned) {
            if let Ok(plan) = serde_json::from_str::<TaskPlan>(&json_str) {
                self.log("Success: Extracted from markdown".to_string());
                return Ok(plan);
            }
        }

        self.log("Attempt 3: Brace-matching JSON extraction".to_string());
        if let Some(json_str) = self.extract_json_object(cleaned) {
            self.log(format!(
                "Found JSON object: {}",
                truncate_str(&json_str, 100)
            ));
            if let Ok(plan) = serde_json::from_str::<TaskPlan>(&json_str) {
                self.log("Success: Brace-matched extraction worked".to_string());
                return Ok(plan);
            }
        }

        Err(format!(
            "Could not parse analysis response: {}",
            truncate_str(cleaned, 100)
        ))
    }

    pub async fn plan_tasks(&mut self, user_message: &str) -> Result<TaskPlan> {
        self.log(format!("Planning tasks for: {}", user_message));

        let session_id = self
            .session_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Orchestrator not initialized"))?;

        let prompt = format!(
            "{}\n\nUser request: {}",
            ORCHESTRATOR_SYSTEM_PROMPT, user_message
        );

        self.log("Sending request to orchestrator AI...".to_string());
        if let Some(ref m) = self.model {
            self.log(format!("Using model: {}", m));
        }
        let response = self
            .server
            .send_message_with_model(&session_id, &prompt, self.model.as_deref())
            .await?;

        let mut full_text = String::new();
        for part in response.parts {
            if let Some(text) = part.text {
                full_text.push_str(&text);
            }
        }

        self.log(format!("Received response ({} chars)", full_text.len()));
        self.log(format!("Raw response: {}", truncate_str(&full_text, 200)));

        match self.parse_task_plan(&full_text, user_message) {
            Ok(plan) => {
                self.log(format!("Successfully parsed {} task(s)", plan.tasks.len()));
                for task in &plan.tasks {
                    self.log(format!("  Task #{}: {}", task.id, task.description));
                }
                Ok(plan)
            }
            Err(e) => {
                self.log(format!("Parse error: {}", e));
                Err(anyhow::anyhow!("{}", e))
            }
        }
    }

    fn parse_task_plan(
        &mut self,
        response: &str,
        original_message: &str,
    ) -> Result<TaskPlan, String> {
        let cleaned = response.trim();

        // Try 1: Direct parse
        self.log("Attempt 1: Direct JSON parse".to_string());
        if let Ok(plan) = serde_json::from_str::<TaskPlan>(cleaned) {
            if !plan.tasks.is_empty() {
                self.log("Success: Direct parse worked".to_string());
                return Ok(plan);
            }
        }

        // Try 2: Extract JSON from markdown code blocks
        self.log("Attempt 2: Extract from markdown code blocks".to_string());
        if let Some(json_str) = self.extract_json_from_markdown(cleaned) {
            if let Ok(plan) = serde_json::from_str::<TaskPlan>(&json_str) {
                if !plan.tasks.is_empty() {
                    self.log("Success: Extracted from markdown".to_string());
                    return Ok(plan);
                }
            }
        }

        // Try 3: Find JSON object with brace matching
        self.log("Attempt 3: Brace-matching JSON extraction".to_string());
        if let Some(json_str) = self.extract_json_object(cleaned) {
            self.log(format!(
                "Found JSON object: {}",
                truncate_str(&json_str, 100)
            ));
            if let Ok(plan) = serde_json::from_str::<TaskPlan>(&json_str) {
                if !plan.tasks.is_empty() {
                    self.log("Success: Brace-matched extraction worked".to_string());
                    return Ok(plan);
                }
            }
        }

        // Try 4: Lenient extraction - find tasks array
        self.log("Attempt 4: Extract tasks array only".to_string());
        if let Some(tasks) = self.extract_tasks_array(cleaned) {
            self.log(format!("Found {} tasks via array extraction", tasks.len()));
            return Ok(TaskPlan {
                tasks,
                reasoning: "Extracted from partial response".to_string(),
                complete: false,
            });
        }

        // Fallback: Create single task from original message
        self.log("All parsing attempts failed, using fallback".to_string());
        self.log(format!(
            "Failed response was: {}",
            truncate_str(cleaned, 200)
        ));

        Ok(TaskPlan {
            tasks: vec![Task {
                id: 1,
                description: truncate_str(original_message, 37),
                prompt: original_message.to_string(),
            }],
            reasoning: format!("Fallback: Could not parse orchestrator response. Executing as single task. Raw response: {}", 
                truncate_str(cleaned, 100)),
            complete: false,
        })
    }

    fn extract_json_from_markdown(&self, text: &str) -> Option<String> {
        // Look for ```json ... ``` or ``` ... ```
        let patterns = ["```json\n", "```json\r\n", "```\n", "```\r\n"];

        for pattern in patterns {
            if let Some(start) = text.find(pattern) {
                let content_start = start + pattern.len();
                if let Some(end) = text[content_start..].find("```") {
                    return Some(text[content_start..content_start + end].trim().to_string());
                }
            }
        }
        None
    }

    fn extract_json_object(&self, text: &str) -> Option<String> {
        let start = text.find('{')?;
        let chars: Vec<char> = text[start..].chars().collect();

        let mut depth = 0;
        let mut end = 0;

        for (i, ch) in chars.iter().enumerate() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end > 0 {
            Some(chars[..end].iter().collect())
        } else {
            None
        }
    }

    fn extract_tasks_array(&self, text: &str) -> Option<Vec<Task>> {
        // Find "tasks": [ ... ]
        let tasks_start = text.find("\"tasks\"")?;
        let array_start = text[tasks_start..].find('[')? + tasks_start;

        let chars: Vec<char> = text[array_start..].chars().collect();
        let mut depth = 0;
        let mut end = 0;

        for (i, ch) in chars.iter().enumerate() {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end > 0 {
            let array_str: String = chars[..end].iter().collect();
            if let Ok(tasks) = serde_json::from_str::<Vec<Task>>(&array_str) {
                if !tasks.is_empty() {
                    return Some(tasks);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_orchestrator() -> Orchestrator {
        Orchestrator {
            server: OpenCodeServer::new(4096),
            session_id: Some("test".to_string()),
            logs: Vec::new(),
            model: None,
        }
    }

    #[test]
    fn test_parse_clean_json() {
        let mut orch = create_test_orchestrator();
        let input = r#"{"tasks": [{"id": 1, "description": "Test", "prompt": "Do test"}], "reasoning": "Simple"}"#;
        let result = orch.parse_task_plan(input, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tasks.len(), 1);
    }

    #[test]
    fn test_parse_markdown_wrapped() {
        let mut orch = create_test_orchestrator();
        let input = r#"Here's my plan:
```json
{"tasks": [{"id": 1, "description": "Test", "prompt": "Do test"}], "reasoning": "Simple"}
```
That's it!"#;
        let result = orch.parse_task_plan(input, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tasks.len(), 1);
    }

    #[test]
    fn test_parse_with_preamble() {
        let mut orch = create_test_orchestrator();
        let input = r#"I'll help you with that. {"tasks": [{"id": 1, "description": "Test", "prompt": "Do test"}], "reasoning": "Simple"} Hope this helps!"#;
        let result = orch.parse_task_plan(input, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tasks.len(), 1);
    }

    #[test]
    fn test_fallback_on_garbage() {
        let mut orch = create_test_orchestrator();
        let input = "I don't understand what you mean, can you clarify?";
        let result = orch.parse_task_plan(input, "original task");
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.tasks[0].prompt, "original task");
    }

    #[test]
    fn test_parse_analysis_complete() {
        let mut orch = create_test_orchestrator();
        let input =
            r#"{"tasks": [], "reasoning": "All tasks completed successfully", "complete": true}"#;
        let result = orch.parse_analysis_response(input);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.tasks.is_empty());
        assert!(plan.complete);
    }

    #[test]
    fn test_parse_analysis_with_followup_tasks() {
        let mut orch = create_test_orchestrator();
        let input = r#"{"tasks": [{"id": 1, "description": "Fix error", "prompt": "Fix the compilation error"}], "reasoning": "Worker 1 failed", "complete": false}"#;
        let result = orch.parse_analysis_response(input);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.tasks.len(), 1);
        assert!(!plan.complete);
        assert_eq!(plan.tasks[0].description, "Fix error");
    }

    #[test]
    fn test_parse_analysis_markdown_wrapped() {
        let mut orch = create_test_orchestrator();
        let input = r#"Here's my analysis:
```json
{"tasks": [], "reasoning": "Done", "complete": true}
```
"#;
        let result = orch.parse_analysis_response(input);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.complete);
    }

    #[test]
    fn test_parse_analysis_fallback() {
        let mut orch = create_test_orchestrator();
        let input = "I'm not sure what to do next.";
        let result = orch.parse_analysis_response(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_worker_result_serialization() {
        let result = WorkerResult {
            worker_id: 1,
            description: "Test task".to_string(),
            success: true,
            output: "Task completed".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("worker_id"));
        assert!(json.contains("success"));
    }

    #[test]
    fn test_task_plan_with_complete_field() {
        let plan = TaskPlan {
            tasks: vec![],
            reasoning: "Done".to_string(),
            complete: true,
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("complete"));
        assert!(json.contains("true"));
    }
}
