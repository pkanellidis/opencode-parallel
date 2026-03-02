# Keyboard Controls Reference

Complete reference for all keyboard shortcuts and controls in opencode-parallel.

## Navigation Mode

The default mode when viewing the TUI.

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `i` or `Enter` | Enter input mode |
| `Esc` | Close logs view / deselect worker |

### Worker Navigation

| Key | Action |
|-----|--------|
| `j` or `↓` | Select next worker |
| `k` or `↑` | Select previous worker |
| `Tab` | Cycle to next worker/session |
| `Shift+Tab` | Cycle to previous worker/session |
| `d` | Delete selected worker (with confirmation) |
| `c` or `C` | Clear all workers (with confirmation) |

### Session Navigation

| Key | Action |
|-----|--------|
| `n` | Next session |
| `p` | Previous session |

### Scrolling

| Key | Action |
|-----|--------|
| `j` or `↓` | Scroll down |
| `k` or `↑` | Scroll up |
| `J` or `PageDown` | Page scroll down |
| `K` or `PageUp` | Page scroll up |
| `g` or `Home` | Scroll to top |
| `G` or `End` | Scroll to bottom |

### View Controls

| Key | Action |
|-----|--------|
| `l` | Toggle orchestrator logs view |

---

## Input Mode

Active when typing messages or commands.

### Text Editing

| Key | Action |
|-----|--------|
| `Enter` | Submit input |
| `Shift+Enter` | Insert newline |
| `Alt+Enter` | Insert newline |
| `Ctrl+Enter` | Insert newline |
| `Esc` | Exit input mode |

### Cursor Movement

| Key | Action |
|-----|--------|
| `←` / `→` | Move cursor left/right |
| `Home` or `Ctrl+A` | Move to line start |
| `End` or `Ctrl+E` | Move to line end |
| `Ctrl+B` | Move cursor back one character |
| `Ctrl+F` | Move cursor forward one character |

### Word Navigation

| Key | Action |
|-----|--------|
| `Ctrl+←` or `Alt+←` | Move to previous word |
| `Ctrl+→` or `Alt+→` | Move to next word |
| `Alt+B` | Move to previous word |
| `Alt+F` | Move to next word |

### Deletion

| Key | Action |
|-----|--------|
| `Backspace` | Delete character before cursor |
| `Delete` or `Ctrl+D` | Delete character at cursor |
| `Ctrl+W` | Delete previous word |
| `Alt+Backspace` | Delete previous word |
| `Ctrl+Backspace` | Delete previous word |
| `Alt+D` | Delete next word |
| `Alt+Delete` | Delete next word |
| `Ctrl+Delete` | Delete next word |
| `Ctrl+K` | Delete from cursor to end of line |
| `Ctrl+U` | Delete from cursor to start of line |
| `Ctrl+C` | Clear entire input |

### Undo/Redo

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+-` | Undo |
| `Ctrl+.` | Redo |

### History (Single-line Input)

| Key | Action |
|-----|--------|
| `↑` | Previous history entry |
| `↓` | Next history entry |

### Autocomplete

| Key | Action |
|-----|--------|
| `↓` | Next suggestion |
| `↑` | Previous suggestion |
| `Tab` or `Enter` | Apply selected suggestion |
| `Esc` | Close autocomplete |

---

## Permission Dialog

When a worker requests file access permission.

| Key | Action |
|-----|--------|
| `←` or `h` | Move selector left |
| `→` or `l` | Move selector right |
| `y` or `Y` | Allow once |
| `a` or `A` | Always allow |
| `n`, `N`, or `Esc` | Reject |
| `Enter` | Confirm current selection |

---

## Model Selector

When choosing an AI model.

| Key | Action |
|-----|--------|
| `↑` or `k` | Move selection up |
| `↓` or `j` | Move selection down |
| `Enter` | Select model |
| `Esc` or `q` | Cancel |

---

## Stop Workers Dialog

When stopping running workers.

| Key | Action |
|-----|--------|
| `↑` or `k` | Move up |
| `↓` or `j` | Move down |
| `Space` | Toggle worker selection |
| `a` or `A` | Select/deselect all |
| `Enter` | Confirm stop |
| `Esc` or `q` | Cancel |

---

## Confirmation Dialogs

For destructive actions (delete, clear).

| Key | Action |
|-----|--------|
| `y` or `Y` | Confirm action |
| `n`, `N`, or `Esc` | Cancel |

---

## Mouse Controls

| Action | Effect |
|--------|--------|
| Scroll wheel up/down | Scroll content |
| Scroll wheel left/right | Horizontal scroll |
| Left-click drag | Select text |
| Release after drag | Auto-copy selection to clipboard |

---

## Slash Commands

Type these in input mode:

| Command | Description |
|---------|-------------|
| `/help` | Show all commands |
| `/new [name]` | Create new session |
| `/sessions` | List all sessions |
| `/rename <name>` | Rename current session |
| `/delete` | Delete current session |
| `/models` | List available models |
| `/model` | Open model selector |
| `/reply #N <msg>` | Reply to worker question |
| `/stop` | Stop running workers |
| `/clear` | Clear session messages |

---

## Terminal Setup

### Option Key as Meta (macOS)

For word-deletion shortcuts (`Alt+Backspace`, `Alt+D`) to work:

**Terminal.app:**
1. Go to Preferences → Profiles → Keyboard
2. Check "Use Option as Meta key"

**iTerm2:**
1. Go to Preferences → Profiles → Keys
2. Set "Left Option Key" to "Esc+"

### Alternative Word Deletion

If `Alt+Backspace` doesn't work, use `Ctrl+W` instead (standard Unix shortcut).

---

## Status Indicators

| Symbol | Meaning |
|--------|---------|
| `○` | Pending (not started) |
| `◉` | Running (in progress) |
| `✓` | Completed (success) |
| `✗` | Failed (error) |
| `⊘` | Cancelled (by user) |
