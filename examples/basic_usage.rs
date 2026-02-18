use opencode_parallel::agent::AgentConfig;

fn main() {
    // Create a new agent
    let mut agent = AgentConfig::new(
        "anthropic",
        "claude-3-5-sonnet-20241022",
        "Refactor authentication module",
    );

    println!("Agent created: {}", agent.id);
    println!("Status: {:?}", agent.status);

    // Start the agent
    agent.start();
    println!("Agent started at: {:?}", agent.started_at);

    // Simulate some output
    agent.add_output("Analyzing authentication module...".to_string());
    agent.add_output("Found 3 areas for improvement".to_string());
    agent.add_output("Applying refactoring...".to_string());

    // Complete the agent
    agent.complete();
    println!("Agent completed at: {:?}", agent.completed_at);

    // Display output
    println!("\nAgent Output:");
    for line in &agent.output {
        println!("  {}", line);
    }

    // Show duration
    if let Some(duration) = agent.duration() {
        println!("\nDuration: {} seconds", duration.num_seconds());
    }
}
