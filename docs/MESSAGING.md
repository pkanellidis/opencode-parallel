# Interactive Messaging with Agents

## Overview

You can now send messages to running opencode agents in real-time! This enables true interactive conversations with your AI agents while they work.

## How It Works

```
┌─────────────────────────────────────────────────────────┐
│  opencode-parallel (TUI)                                │
│                                                          │
│  User presses 'w' → Types message → Presses Enter       │
│                            │                            │
│                            ▼                            │
│                   Write to stdin                        │
└────────────────────────────┬────────────────────────────┘
                             │
                             │ "Can you explain more?\n"
                             │
                             ▼
                ┌─────────────────────────┐
                │   opencode process      │
                │   (running agent)       │
                │                         │
                │   Receives message      │
                │   Processes it          │
                │   Generates response    │
                └────────┬────────────────┘
                         │
                         │ Response text
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│  opencode-parallel (TUI)                                │
│                                                          │
│  Captures stdout → Displays in output pane              │
└─────────────────────────────────────────────────────────┘
```

## Usage

### 1. Start an Agent

```bash
./target/release/opencode-parallel tui --agents 4
```

### 2. Select a Running Agent

Use `↑`/`k` or `↓`/`j` to select an agent that shows:
- `▶` Running status (yellow)
- Active progress bar

### 3. Enter Write Mode

Press `w` to enter write mode. You'll see:

```
╭─────────────────────────────────────────────────────────────────╮
│ 💬 Message: █  [Enter] Send [Esc] Cancel                       │
╰─────────────────────────────────────────────────────────────────╯
```

### 4. Type Your Message

Type your question or instruction:

```
╭─────────────────────────────────────────────────────────────────╮
│ 💬 Message: Can you explain that in more detail?█              │
╰─────────────────────────────────────────────────────────────────╯
```

### 5. Send It

Press `Enter` to send. Your message will:
1. Appear in the output pane with a magenta `→` prefix
2. Be sent to opencode's stdin
3. Get processed by the AI agent
4. Generate a response that streams back

## Example Interaction

### Scenario: Asking for Clarification

1. Agent is explaining Rust ownership
2. You press `w`
3. Type: "Can you give a concrete example?"
4. Press `Enter`

**What you'll see:**

```
╭───────── 💬 Output (scroll: 0/25) ─────────╮
│ 🚀 Starting opencode for: Explain Rust    │
│ 📦 Model: anthropic / claude-3-5...       │
│ Rust's ownership system ensures memory... │
│ Three main rules:                         │
│ 1. Each value has an owner                │
│ 2. Only one owner at a time               │
│ → You: Can you give a concrete example?   │
│ Certainly! Here's a concrete example...   │
│ ```rust                                    │
│ let s1 = String::from("hello");           │
│ let s2 = s1; // s1 is moved to s2         │
│ ```                                        │
╰────────────────────────────────────────────╯
```

## Technical Details

### Message Delivery

When you send a message:

```rust
// 1. Your message is captured
let message = "Can you explain more?";

// 2. Added to output for display
agent.add_output(format!("→ You: {}", message));

// 3. Sent to opencode's stdin
stdin.write_all(message.as_bytes()).await?;
stdin.write_all(b"\n").await?;  // Add newline
stdin.flush().await?;             // Ensure delivery
```

### Process Communication

```
┌──────────────────────────────────────────────┐
│ opencode-parallel                            │
│                                              │
│  ┌─────────────────────────────────────┐   │
│  │ ChildStdin handle                   │   │
│  │ (stored per agent)                  │   │
│  └──────────────┬──────────────────────┘   │
│                 │                            │
│                 │ write_all() + flush()      │
│                 ▼                            │
│  ┌─────────────────────────────────────┐   │
│  │ opencode process                    │   │
│  │ stdin → processes → stdout          │   │
│  └─────────────┬───────────────────────┘   │
│                │                            │
│                │ BufReader.lines()          │
│                ▼                            │
│  ┌─────────────────────────────────────┐   │
│  │ Output display                       │   │
│  └─────────────────────────────────────┘   │
└──────────────────────────────────────────────┘
```

## Limitations & Notes

### When You CAN Send Messages

✅ Agent status is **Running** (▶)
✅ Stdin handle is available
✅ Process hasn't completed yet

### When You CANNOT Send Messages

❌ Agent is **Pending** (⏸) - hasn't started
❌ Agent is **Completed** (✓) - process ended
❌ Agent is **Failed** (✗) - process crashed
❌ Agent is **Cancelled** (⊘) - you stopped it

### Message Behavior

- **Newline added**: Each message gets `\n` automatically
- **Flushed immediately**: No buffering delays
- **Displayed locally**: Message shows in output pane
- **Processed remotely**: opencode receives and responds

### Error Handling

If message sending fails:

```
✗ Failed to send message: Broken pipe (os error 32)
```

This usually means:
- Process has terminated
- Stdin was closed
- Pipe was broken

## Use Cases

### 1. Asking Follow-up Questions

```
Agent: "Here's how async/await works..."
You: "Can you show error handling too?"
Agent: "Certainly! Here's how to handle errors..."
```

### 2. Providing Additional Context

```
Agent: "I need more information to help..."
You: "The error occurs when connecting to PostgreSQL"
Agent: "Ah, let me help with PostgreSQL connection..."
```

### 3. Refining Output

```
Agent: "Here's a solution using recursion..."
You: "Can you make it iterative instead?"
Agent: "Sure! Here's the iterative version..."
```

### 4. Correcting Course

```
Agent: "I'll implement this using arrays..."
You: "Actually, please use HashMaps"
Agent: "Understood, switching to HashMap..."
```

## Tips

### 💡 Best Practices

1. **Be Clear**: Short, specific messages work best
2. **Wait for Response**: Let agent finish before sending more
3. **Check Status**: Only send to running agents (▶)
4. **Use Scroll**: Press `u`/`d` to see full conversation

### ⚡ Shortcuts

- `w` → Quick write mode entry
- `Esc` → Cancel without sending
- `Backspace` → Fix typos before sending
- `Enter` → Send and exit write mode

### 🎨 Visual Cues

- **Magenta text**: Your messages stand out
- **→ prefix**: Easy to spot in output
- **Cursor █**: Shows you're in write mode
- **Green [Enter]**: Reminds you how to send

## Comparison with opencode CLI

### opencode CLI (Direct)
```bash
opencode run "Explain Rust"
# You can't interrupt or ask questions
# One-way communication
```

### opencode-parallel (Interactive)
```bash
opencode-parallel tui
# Select agent, press 'w', type message
# Two-way conversation
# Real-time interaction
```

## Future Enhancements

Possible improvements:
- [ ] Message history (up/down arrows)
- [ ] Autocomplete for common messages
- [ ] Batch messages to multiple agents
- [ ] Save conversation transcripts
- [ ] Template messages (e.g., "explain more", "add tests")
- [ ] Copy/paste support
- [ ] Multi-line messages

## Troubleshooting

### Message not appearing?

Check:
1. Is agent status **Running** (▶)?
2. Did you press `Enter` after typing?
3. Is the output scrolled down to see latest?

### No response from agent?

- opencode might be processing
- Wait a few seconds
- Check if process is still running
- Look for error messages

### Can't enter write mode?

- Press `w` (not 'W')
- Ensure agent is selected (highlighted)
- Make sure agent is running (▶)

## Example Session

```
1. Start: opencode-parallel tui --agents 2

2. Agent 1 starts explaining Rust
   Output: "Rust is a systems programming language..."

3. Press 'j' to select Agent 1
   Selection: ▶ Agent 1 (5s) [████░░░]

4. Press 'w'
   Footer: 💬 Message: █

5. Type: "show me code"
   Footer: 💬 Message: show me code█

6. Press Enter
   Output: → You: show me code
   Output: Certainly! Here's an example:
   Output: ```rust
   Output: fn main() { ... }

7. Continue conversation as needed!
```

---

**Now your messages actually reach opencode!** 🎉

Try it: `./target/release/opencode-parallel tui --agents 4`
