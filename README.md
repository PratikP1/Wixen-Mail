# Wixen-Mail
A fully accessible, light-weight, open source mail client using Rust and WXDragon. The client aims to be fast and look similar to Thunderbird.

## Project Scope
- Platform focus: Windows first
- UI stack: WXDragon
- Product direction: Thunderbird-like workflow with modern accessibility defaults

## Accessibility Baseline (Required)
Wixen Mail must be fully usable without a mouse and readable with screen readers (NVDA, JAWS, Narrator).

Planned keyboard commands:
- `Ctrl+N`: Compose new mail
- `Ctrl+R`: Reply
- `Ctrl+Shift+R`: Reply all
- `Ctrl+F`: Forward
- `Ctrl+Enter`: Send message
- `Ctrl+1`: Focus folder pane
- `Ctrl+2`: Focus message list
- `Ctrl+3`: Focus message preview
- `F6` / `Shift+F6`: Move forward/backward between major regions
- `Tab` / `Shift+Tab`: Move between interactive controls
- `Delete`: Delete selected message
- `Ctrl+Shift+A`: Mark all as read in current folder

See `TASKS.md` for the implementation checklist.
