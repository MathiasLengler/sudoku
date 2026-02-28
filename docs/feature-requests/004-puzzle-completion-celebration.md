# 004: Puzzle Completion Celebration

**Priority:** P2 – Medium  
**Dependencies:** None (optionally enhanced by [001-game-timer.md](001-game-timer.md))

## Summary

Add a visually rewarding celebration effect when the player successfully solves a puzzle. Currently, the only feedback is a small text notification ("Sudoku is solved") displayed in the hint button area. A more prominent celebration improves the sense of accomplishment.

## Why It Is Useful

- **Player satisfaction:** Completing a puzzle is the climactic moment of gameplay. A muted response feels anticlimactic, especially after spending significant time on a hard puzzle.
- **Emotional engagement:** Visual celebrations (confetti, animations, modals) create positive emotional associations that encourage the player to start another game.
- **Industry standard:** Nearly all modern puzzle apps include celebration effects. Their absence makes the app feel incomplete.
- **New game funnel:** A completion modal is a natural place to show the solve time (if [001-game-timer.md](001-game-timer.md) is implemented), offer to start a new game, or share the result.

## Current State

- `sudokuIsSolvedState` atom correctly detects puzzle completion.
- A success `Snackbar` notification with text "Sudoku is solved" is displayed via `RequestHintButton.tsx`.
- No animations, modals, or visual effects exist on completion.
- No completion statistics are shown.

## Proposed Implementation

1. **Completion dialog** (`PuzzleCompleteDialog.tsx`):
   - Modal dialog that appears when `sudokuIsSolvedState` becomes `true`.
   - Shows: "Congratulations! 🎉", solve time (if available), difficulty rating (if available).
   - Action buttons: "New Game" (opens `NewGameDialog`), "Share" (copy result), "Close".
   - Uses MUI `Dialog` component consistent with existing dialogs.

2. **Confetti animation** (optional, lightweight):
   - Use a small library like `canvas-confetti` (~3KB gzipped) for a brief confetti burst.
   - Trigger on dialog open; runs for 2-3 seconds.
   - Respect `prefers-reduced-motion` media query — skip animation if the user prefers reduced motion.

3. **Grid animation:**
   - Brief highlight/pulse animation on all cells when solved.
   - Use CSS `@keyframes` animation, no external library needed.
   - Cells briefly flash green or gold, then settle.

4. **Remove existing snackbar:** Replace the text notification with the new celebration dialog.

## Challenges and Risks

| Challenge | Mitigation |
|-----------|------------|
| **Annoyance factor:** Players solving many easy puzzles may find celebrations disruptive | Add a "Don't show again" checkbox or a setting to disable celebrations. |
| **Bundle size:** Adding a confetti library increases the bundle | `canvas-confetti` is ~3KB gzipped; negligible. Alternatively, implement a minimal CSS-only animation. |
| **Reduced motion:** Animations can be problematic for users with vestibular disorders | Check `prefers-reduced-motion` and skip animations entirely if set. |
| **Undo after solve:** Player may undo the last move, un-solving the puzzle | Dismiss the dialog if `sudokuIsSolvedState` becomes `false`. Do not block further interaction. |
| **World mode:** Completion detection may differ for multi-grid world puzzles | Initially scope celebration to standard single-grid mode only. |

## Testing Approach

- **Component tests:**
  - Render `PuzzleCompleteDialog` with `sudokuIsSolvedState = true` and verify it appears.
  - Verify "New Game" button opens the new game dialog.
  - Verify dialog dismisses when "Close" is clicked.
  - Verify dialog dismisses if puzzle becomes unsolved (undo scenario).
- **Accessibility tests:**
  - Verify dialog traps focus correctly (MUI `Dialog` does this by default).
  - Verify dialog is announced by screen readers.
  - Verify confetti respects `prefers-reduced-motion`.
- **Visual regression tests:**
  - Screenshot comparison of the celebration dialog in light and dark themes.
  - Verify no layout shifts when the dialog appears.
- **Integration tests:**
  - Solve a puzzle end-to-end and verify the celebration dialog appears.
  - Start a new game from the dialog and verify the timer resets (if timer is implemented).
