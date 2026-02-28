# 006: Enhanced Keyboard Shortcuts

**Priority:** P2 – Medium  
**Dependencies:** Partially related to [003-accessibility-improvements.md](003-accessibility-improvements.md)

## Summary

Expand the keyboard shortcut set to include standard undo/redo bindings (`Ctrl+Z` / `Ctrl+Y` / `Ctrl+Shift+Z`), additional navigation shortcuts, and a discoverable shortcuts help dialog. The existing keyboard handler has a TODO comment for `Shift+Backspace` redo but it is not implemented.

## Why It Is Useful

- **Platform conventions:** `Ctrl+Z` / `Cmd+Z` for undo and `Ctrl+Y` / `Cmd+Shift+Z` for redo are universal conventions that users expect in any interactive application.
- **Discoverability:** Currently, keyboard shortcuts (arrow keys, Space, Insert, etc.) are undocumented in the UI. A shortcuts help dialog makes them discoverable.
- **Power users:** Experienced players benefit from rapid keyboard-driven gameplay without touching the mouse or screen.
- **Existing code gap:** The keyboard handler explicitly has a `TODO` for redo support (`Shift+Backspace`, `Ctrl+Z`, `Ctrl+Y`), indicating this was already planned.

## Current State

Implemented shortcuts (in `useKeyboardInput.tsx`):
- Arrow keys: cell navigation
- Number keys (0-9, A-F): value entry
- Space: toggle candidate mode
- `+`: toggle sticky mode
- Delete: clear cell
- Insert: set all direct candidates
- Backspace: undo

**Not implemented:**
- `Shift+Backspace`: redo (marked as TODO)
- `Ctrl+Z` / `Cmd+Z`: undo (standard binding)
- `Ctrl+Y` / `Cmd+Y` / `Ctrl+Shift+Z` / `Cmd+Shift+Z`: redo (standard binding)
- `Ctrl+Shift+Z`: redo (macOS convention)
- `?` or `F1`: shortcuts help dialog
- `N`: new game shortcut
- `H`: request hint shortcut
- `Escape`: deselect cell / close dialog

## Proposed Implementation

1. **Extend `useKeyboardInput.tsx`:**
   ```typescript
   // Redo bindings
   case "z": if (e.ctrlKey || e.metaKey) { e.shiftKey ? redo() : undo(); } break;
   case "y": if (e.ctrlKey || e.metaKey) { redo(); } break;
   // Convenience shortcuts
   case "Escape": deselectCell(); break;
   case "?": openShortcutsDialog(); break;
   case "h": requestHint(); break;
   case "n": openNewGameDialog(); break;
   ```

2. **Shortcuts help dialog** (`ShortcutsHelpDialog.tsx`):
   - Opened via `?` key or a help icon button in the app bar.
   - Displays a formatted table of all available shortcuts.
   - Groups shortcuts by category: Navigation, Input, Actions, Modes.
   - Uses MUI `Dialog` and `Table` components.

3. **Platform-aware display:**
   - Show `Cmd` on macOS/iOS, `Ctrl` on Windows/Linux.
   - Use `navigator.platform` or `navigator.userAgentData.platform` for detection.

4. **Prevent default behavior:**
   - `Ctrl+Z`, `Ctrl+Y` must call `e.preventDefault()` to prevent browser-native undo in text fields.

## Challenges and Risks

| Challenge | Mitigation |
|-----------|------------|
| **Browser shortcut conflicts:** `Ctrl+Y` opens browser history in some browsers | Call `e.preventDefault()` and `e.stopPropagation()` for handled shortcuts. Most browsers allow this override for non-navigation shortcuts. |
| **macOS vs Windows:** Different modifier keys | Use `e.metaKey` for macOS (`Cmd`) and `e.ctrlKey` for Windows/Linux. Both should be checked. |
| **Dialog interference:** Shortcuts should not fire when a dialog is open | Check if any MUI `Dialog` is open before processing shortcuts, or stop propagation in dialog event handlers. |
| **Mobile irrelevance:** Keyboard shortcuts are useless on touch devices | Only show the shortcuts help button when a physical keyboard is detected (e.g., `matchMedia("(hover: hover)")`). |
| **Shortcut collisions:** New shortcuts may conflict with existing ones | Audit all current shortcuts before adding new ones. Use modifier keys for new actions. |

## Testing Approach

- **Unit tests:**
  - Simulate `Ctrl+Z` keydown event and verify `undo()` is called.
  - Simulate `Ctrl+Shift+Z` and verify `redo()` is called.
  - Simulate `Ctrl+Y` and verify `redo()` is called.
  - Simulate `Escape` and verify cell is deselected.
  - Simulate `?` and verify shortcuts dialog opens.
  - Verify `e.preventDefault()` is called for handled shortcuts.
- **Cross-platform tests:**
  - Simulate `Meta+Z` (macOS) and verify undo works.
  - Verify shortcut display shows `Cmd` on macOS and `Ctrl` on Windows.
- **Dialog tests:**
  - Render `ShortcutsHelpDialog` and verify all shortcuts are listed.
  - Verify shortcuts are grouped by category.
  - Verify dialog closes with `Escape`.
- **Integration tests:**
  - Open the app, make moves, press `Ctrl+Z` multiple times, verify undo chain.
  - Verify `Ctrl+Y` redoes undone moves.
  - Verify shortcuts do not fire when a dialog (e.g., NewGameDialog) is open.
