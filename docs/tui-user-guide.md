## Terminal UI (TUI)

The opencode-parallel TUI provides a rich terminal interface for managing parallel AI coding workers. It features a multi-panel layout, vim-inspired keybindings, slash commands, session management, and interactive dialogs.

### Layout

The TUI uses a three-panel layout that adapts based on context:

| Panel | Position | Description |
|-------|----------|-------------|
| **Workers Sidebar** | Left (30 chars wide) | Lists all workers with status icons. Visible when workers exist. |
| **Messages Panel** | Center | Shows chat history, task plans, worker summaries, and system messages. Scrollable with scrollbar. |
| **Worker Detail Panel** | Right (45% of remaining width) | Shows selected worker's tool calls, streaming output, and status. Appears when a worker is selected. |
| **Input Box** | Bottom (sticky) | Multiline text input area. Dynamic height (4-12 lines) based on content. |
| **Landing View** | Center | Shown when no workers exist. Displays welcome message and instructions. |
| **Logs Panel** | Full overlay | Orchestrator debug logs. Toggled with `l` key. |

The status bar at the bottom of the input box shows the current model on the left and context-sensitive hints on the right.

---

### Slash Commands

All commands are case-insensitive and start with `/`. Typing `/` in input mode activates autocomplete.

| Command | Aliases | Arguments | Description |
|---------|---------|-----------|-------------|
| `/help` | `/h`, `/?`, `/` | None | Show available commands |
| `/new` | `/n` | `[name]` (optional) | Create a new session with optional name |
| `/sessions` | `/ls` | None | List all sessions with worker counts |
| `/rename` | `/mv` | `<name>` (required) | Rename the current session |
| `/delete` | `/del`, `/rm` | None | Delete the current session (with confirmation) |
| `/models` | None | None | List connected providers |
| `/model` | `/m` | None | Open interactive model selector |
| `/model` | `/m` | `<provider/model>` | Set model directly (e.g., `/model openai/gpt-4`) |
| `/reply` | `/r` | `#N [message]` | Reply to worker question or continue conversation |
| `/stop` | `/s`, `/kill`, `/cancel` | None | Open stop worker selector to interrupt running workers |
| `/projects` | `/project`, `/proj`, `/p` | None | List all projects |
| `/project current` | `/proj current`, `/p current` | None | Show current project |
| `/path` | `/pwd` | None | Show current working directory path |
| `/clear` | `/cls` | None | Clear chat messages in current session |
| `/config` | `/cfg` | None | Show current server configuration (JSON) |

**Reply command details:**
- `/reply #1 yes` - Reply "yes" to worker #1's pending question
- `/reply #1` - Continue worker #1's conversation (requires a message for non-question workers)
- `/r #3 ok` - Short form reply to worker #3
- The `#` prefix on the worker number is optional: `/reply 1 yes` also works

---

### Keyboard Navigation

#### Global (Any Mode)

| Key | Action |
|-----|--------|
| `Ctrl+C` | Copy selected text to clipboard (when text is selected) |

#### Navigation Mode (default when not typing)

**General:**

| Key | Action |
|-----|--------|
| `q` | Quit the application |
| `i` / `Enter` | Enter input mode (when not in detail panel) |
| `l` | Toggle orchestrator logs panel |
| `Esc` | Close logs panel / unfocus detail panel / deselect worker (cascading) |

**Worker Selection:**

| Key | Action |
|-----|--------|
| `j` / `Down` | Select next worker (or scroll if in detail/logs panel) |
| `k` / `Up` | Select previous worker (or scroll if in detail/logs panel) |
| `d` | Delete selected worker (shows confirmation) |
| `c` / `C` | Clear all workers (shows confirmation) |

**Panel Navigation:**

| Key | Action |
|-----|--------|
| `Tab` | Toggle focus between workers sidebar and detail panel (if worker selected); otherwise switch to next session |
| `Shift+Tab` | Same as Tab (toggle panel focus or switch to previous session) |
| `h` / `Left` | Unfocus detail panel (return to sidebar) |
| `Right` | Focus detail panel (when worker is selected) |

**Session Switching:**

| Key | Action |
|-----|--------|
| `n` | Switch to next session |
| `p` | Switch to previous session |
| `Tab` | Next session (when no worker is selected) |
| `Shift+Tab` | Previous session (when no worker is selected) |

**Scrolling:**

| Key | Action |
|-----|--------|
| `j` / `Down` | Scroll down 1 line (in detail or logs panel) |
| `k` / `Up` | Scroll up 1 line (in detail or logs panel) |
| `J` / `PageDown` | Scroll down 20 lines (page) / 10 lines in main panel |
| `K` / `PageUp` | Scroll up 20 lines (page) / 10 lines in main panel |
| `g` / `Home` | Scroll to top |
| `G` / `End` | Scroll to bottom |

Scroll context is determined by focus: logs panel > detail panel > main messages panel.

#### Input Mode

**Submission & Control:**

| Key | Action |
|-----|--------|
| `Enter` | Submit input (send message or execute command) |
| `Shift+Enter` / `Alt+Enter` / `Ctrl+Enter` | Insert newline (multiline input) |
| `Esc` | Exit input mode (return to navigation) |
| `Ctrl+C` | Clear input text |

**Cursor Movement:**

| Key | Action |
|-----|--------|
| `Ctrl+A` / `Home` | Move cursor to beginning of line |
| `Ctrl+E` / `End` | Move cursor to end of line |
| `Ctrl+B` | Move cursor back one character |
| `Ctrl+F` | Move cursor forward one character |
| `Ctrl+Left` / `Alt+Left` / `Alt+B` | Move cursor back one word |
| `Ctrl+Right` / `Alt+Right` / `Alt+F` | Move cursor forward one word |
| `Up` | Navigate to previous history entry (single-line mode) |
| `Down` | Navigate to next history entry (single-line mode) |

**Deletion:**

| Key | Action |
|-----|--------|
| `Ctrl+K` | Delete from cursor to end of line |
| `Ctrl+U` | Delete from cursor to beginning of line |
| `Ctrl+W` / `Ctrl+Backspace` / `Alt+Backspace` | Delete previous word |
| `Alt+D` / `Ctrl+Delete` / `Alt+Delete` | Delete next word |
| `Ctrl+D` | Delete character under cursor |

**Undo/Redo:**

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` / `Ctrl+.` | Redo |
| `Ctrl+-` | Undo (alternative) |

**Autocomplete (when typing `/` commands):**

| Key | Action |
|-----|--------|
| `Up` | Previous suggestion |
| `Down` | Next suggestion |
| `Tab` / `Enter` | Apply selected suggestion |
| `Esc` | Close autocomplete popup |

Autocomplete activates automatically when input starts with `/` and shows matching commands with descriptions.

#### Permission Dialog

| Key | Action |
|-----|--------|
| `y` / `Y` | Allow once |
| `a` / `A` | Allow always |
| `n` / `N` / `Esc` | Reject |
| `Left` / `h` | Move selector left |
| `Right` / `l` | Move selector right |
| `Enter` | Confirm current selection |

#### Model Selector Dialog

| Key | Action |
|-----|--------|
| `j` / `Down` | Next model |
| `k` / `Up` | Previous model |
| `Enter` | Select model |
| `q` / `Esc` | Cancel |

#### Stop Worker Selector Dialog

| Key | Action |
|-----|--------|
| `j` / `Down` | Move cursor down |
| `k` / `Up` | Move cursor up |
| `Space` | Toggle selection on current worker |
| `a` / `A` | Select/deselect all workers |
| `Enter` | Stop selected workers |
| `q` / `Esc` | Cancel |

#### Confirmation Dialogs (Delete Worker, Clear All, Delete Session)

| Key | Action |
|-----|--------|
| `y` / `Y` | Confirm |
| `n` / `N` / `Esc` | Cancel |

---

### Session Management

Sessions group workers and conversations together. Each session maintains its own message history, worker list, and orchestrator context.

**Creating sessions:**
- `/new` creates a session with an auto-generated name (e.g., "Session 2")
- `/new My Feature` creates a session named "My Feature"
- New sessions are immediately made active

**Switching sessions:**
- `n` key (navigation mode) cycles to the next session
- `p` key (navigation mode) cycles to the previous session
- `Tab` / `Shift+Tab` cycles sessions when no worker is selected
- Sessions cycle (wrapping from last to first and vice versa)

**Renaming sessions:**
- `/rename New Name` renames the current session

**Listing sessions:**
- `/sessions` shows all sessions with a `>` marker on the current one, plus worker counts

**Deleting sessions:**
- `/delete` triggers a confirmation dialog (`y`/`n`)
- The last remaining session cannot be deleted
- Deletion adjusts the current session index if needed

**Session persistence:**
- Each session preserves its orchestrator session ID, allowing follow-up requests to build on previous context

---

### Worker Interaction

Workers are parallel task executors spawned by the orchestrator. Each runs in its own OpenCode session.

**Worker states:**

| Symbol | State | Description |
|--------|-------|-------------|
| `◌` / `◐` | Starting | Worker is initializing |
| `●` | Running | Worker is actively processing |
| `?` | WaitingForInput | Worker has asked a question |
| `✓` | Complete | Worker finished successfully |
| `✗` | Error | Worker encountered an error |

**Viewing workers:**
- The sidebar shows all workers with state icons, IDs, and truncated descriptions
- Select workers with `j`/`k` or `Up`/`Down` keys
- The detail panel shows full tool call history, streaming output, and status
- Tool calls display with headers, parameters, and results

**Replying to worker questions:**
- When a worker asks a question, the message panel shows the question and a hint: `Reply: /reply #N <answer>`
- Use `/reply #N answer text` to respond
- The worker resumes processing after receiving the reply

**Continuing worker conversations:**
- `/reply #N message` can send follow-up messages to any worker (not just those with pending questions)
- The worker must have an active OpenCode session

**Stopping workers:**
- `/stop` opens a multi-select dialog listing running workers
- Use `Space` to toggle selection, `a` to select all
- `Enter` confirms and stops selected workers (marks them as Error with "Interrupted by user")

**Deleting workers:**
- Select a worker and press `d` to open deletion confirmation
- Press `c`/`C` to clear all workers (with confirmation)

---

### Permission System

When a worker's OpenCode session requests a permission (e.g., file edit, bash command), a permission dialog appears automatically.

**Dialog contents:**
- Source: Which worker is requesting (with ID and description)
- Tool: The permission type (e.g., "edit", "bash")
- Files: Affected file patterns (up to 3 shown, with "and N more" for extras)
- Pending count: Shows how many more permission requests are queued

**Response options:**
- **Yes** (`y`): Allow this specific operation once
- **Always** (`a`): Allow this type of operation permanently for the session
- **Reject** (`n` / `Esc`): Deny the operation

Use arrow keys or `h`/`l` to navigate between options, or press the shortcut key directly. Multiple queued permissions are handled sequentially.

---

### Text Selection & Clipboard

**Mouse selection:**
- Click and drag to select text in the messages panel or detail panel
- Selection highlights text with a distinct background color
- Releasing the mouse button automatically copies selected text to the clipboard
- Both the system clipboard (via `arboard`) and OSC 52 (for SSH/tmux support) are used

**Keyboard copy:**
- `Ctrl+C` copies the current selection to the clipboard (when text is selected)
- Status bar confirms with "Copied N chars"

**Panel-aware selection:**
- Selection automatically detects which panel (messages or detail) the mouse is over
- Each panel tracks its own content lines for accurate text extraction

**Scroll-aware:**
- Mouse scroll events are routed to the panel under the cursor
- Supports scroll in all four directions (up, down, left, right)

---

### Input Area

The input area at the bottom of the screen is an enhanced multiline text editor with opencode-style keybindings.

**Features:**
- **Multiline editing**: Use `Shift+Enter`, `Alt+Enter`, or `Ctrl+Enter` to insert newlines
- **Word wrap**: Text wraps at word boundaries
- **Dynamic height**: Input box grows from 4 to 12 lines based on content
- **Input history**: `Up`/`Down` arrows cycle through previous inputs (single-line mode only)
- **Slash command autocomplete**: Type `/` to see matching commands; navigate with `Up`/`Down`, select with `Tab`/`Enter`
- **Undo/Redo**: Full undo/redo support with `Ctrl+Z` and `Ctrl+Shift+Z`
- **Emacs-style navigation**: `Ctrl+A/E/B/F/K/U/W/D` keybindings
- **Word navigation**: `Alt+B/F` or `Ctrl+Left/Right` for word-level movement
- **Bracketed paste**: Pasting multiline text preserves newlines without triggering submission
- **Placeholder text**: Shows "Press 'i' to enter a task..." when empty and not focused

**History:**
- Submitted inputs are saved to history (duplicates and blank entries are skipped)
- Navigate with `Up`/`Down` when on a single line
- Current unsaved input is preserved when navigating history

---

### Scrolling

Scrolling uses macOS-style acceleration with momentum:

- **Mouse wheel**: Scroll events have velocity tracking and acceleration curves
- **Keyboard**: `j`/`k` for single lines, `J`/`K`/`PageUp`/`PageDown` for pages, `g`/`G`/`Home`/`End` for top/bottom
- **Position-aware**: Mouse scroll events route to the panel under the cursor
- **Auto-scroll**: Worker detail panel auto-scrolls to bottom while a worker is running
- **Scrollbar**: Visible when content exceeds the viewport
