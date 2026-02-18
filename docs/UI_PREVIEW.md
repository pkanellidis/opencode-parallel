# UI Preview - Enhanced Interface

## New Visual Design

```
╔═══════════════════════════════════════════════════════════╗
║  OPENCODE PARALLEL - Multiple AI Agents in Parallel  ║
╚═══════════════════════════════════════════════════════════╝

┌─────────── 🤖 Agents ───────────┬──────────── 📊 Agent 2 - Explain Rust ─────────────┐
│                                  │                                                     │
│  ▶ Agent 1 (5s) [████████░░░]  │  ╭─────────────────────────────────────────────────╮│
│  ▶ Agent 2 (12s) [█████░░░░░░] │  │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░ 75%                    ││
│  ✓ Agent 3 (8s)                 │  ╰─────────────────────────────────────────────────╯│
│  ✗ Agent 4 (3s)                 │                                                     │
│  ⏸ Agent 5                      │  ╭───────── 💬 Output (scroll: 0/25) ─────────────╮│
│  ⏸ Agent 6                      │  │ 🚀 Starting opencode for: Explain Rust         ││
│                                  │  │ 📦 Model: anthropic / claude-3-5-sonnet-202... ││
│                                  │  │ Analyzing the question...                      ││
│                                  │  │ Rust is a systems programming language...      ││
│                                  │  │ Key features:                                  ││
│                                  │  │ - Memory safety without garbage collection     ││
│                                  │  │ - Zero-cost abstractions                       ││
│                                  │  │ → You: Can you explain ownership?              ││
│                                  │  │ Certainly! Ownership in Rust means...          ││
│                                  │  │ ✓ Task completed successfully!                 ││
│                                  │  ╰─────────────────────────────────────────────────╯│
└──────────────────────────────────┴─────────────────────────────────────────────────────┘

╭─────────────────────────────────────────────────────────────────────────────────────╮
│ q:Quit ↑/k:Up ↓/j:Down s:Start c:Cancel w:Write u/d:Scroll                         │
╰─────────────────────────────────────────────────────────────────────────────────────╯
```

## Color Scheme

### Status Icons
- `▶` **Running** - Yellow (indicates active processing)
- `✓` **Complete** - Green (successful completion)
- `✗` **Failed** - Red (error occurred)
- `⏸` **Pending** - Gray (waiting to start)
- `⊘` **Cancelled** - Dark Gray (user cancelled)

### Progress Bars
- **Active agents**: Yellow progress bars with animated blocks
- **Completed agents**: Green at 100%
- **Failed agents**: Red (stuck at failure point)
- **Format**: `[████████░░░]` (filled/empty blocks)

### Message Types
- **User messages**: Magenta with `→ You:` prefix
- **Success messages**: Green (lines starting with ✓)
- **Error messages**: Red (lines starting with ✗)
- **System messages**: Default color

### UI Elements
- **Header**: Cyan, bold, ASCII art box
- **Borders**: Rounded borders with color coding
  - Agent list: Cyan borders
  - Output area: Yellow borders
  - Footer: Gray borders
- **Selected agent**: Cyan text with dark gray background

## Write Mode Interface

When you press 'w' to write a message:

```
╭─────────────────────────────────────────────────────────────────────────────────────╮
│ 💬 Message: Hello, can you help with this?█  [Enter] Send [Esc] Cancel            │
╰─────────────────────────────────────────────────────────────────────────────────────╯
```

- Magenta prompt with message icon
- Real-time typing with cursor (█)
- Clear action hints in green/red

## Progress Visualization

### Calculation
Progress is estimated based on:
- **0%**: Agent pending (not started)
- **0-95%**: Based on output lines (1 line ≈ 1%)
- **100%**: Agent completed successfully

### Display Formats
```
Agent 1 (12s) [████████████████████]  ← 100% (completed)
Agent 2 (8s) [████████████░░░░░░░░]   ← 60% (in progress)
Agent 3 (5s) [██░░░░░░░░░░░░░░░░░░]   ← 10% (just started)
Agent 4                                ← 0% (pending)
```

## Interactive Features

### Agent Selection
- Selected agent has cyan background
- Selection follows vim keybindings (j/k)
- Scroll position resets when changing agents

### Output Scrolling
- Shows current scroll position: `(scroll: 10/50)`
- 'u' to scroll up, 'd' to scroll down
- Auto-fits to terminal height

### Real-time Updates
- Output streams live from opencode processes
- Progress bars update automatically
- Duration counters tick every second
- Status changes reflect immediately

## Responsive Layout

### Horizontal Split
- **Left pane (35%)**: Agent list with status
- **Right pane (65%)**: Selected agent details
  - Top section: Progress gauge (3 lines)
  - Bottom section: Output with scroll

### Vertical Sections
- **Header (3 lines)**: Title banner
- **Main (dynamic)**: Split panes
- **Footer (3 lines)**: Help text / input

## Terminal Requirements

- **Minimum size**: 80x24 characters
- **Color support**: 256 colors recommended
- **Unicode support**: For icons and progress bars
- **Cursor**: Hidden during normal mode, shown in write mode

## Accessibility

- Clear visual hierarchy
- High contrast colors
- Meaningful icons with text labels
- Keyboard-only navigation
- Status always visible

## Performance

- Updates at ~10 FPS (100ms poll interval)
- Efficient rendering (only changed regions)
- No flickering with double buffering
- Smooth scrolling and transitions

## Future Enhancements

Possible additions:
- Mouse support for clicking agents
- Themes (dark/light/custom colors)
- Customizable keybindings
- Split screen for multiple agent views
- Graph view of agent relationships
- Log export per agent
- Search/filter in output

---

**Try it out:**
```bash
./target/release/opencode-parallel tui --agents 4
```

Press 'w' on any running agent to send it a message!
