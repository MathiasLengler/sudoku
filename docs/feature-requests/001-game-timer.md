# 001: Game Timer / Solve Time Tracking

**Priority:** P1 – High  
**Dependencies:** None

## Summary

Add a visible timer that tracks how long a player takes to solve a puzzle, displayed in the app bar or toolbar area. The timer starts when a new puzzle is loaded and stops when the puzzle is solved.

## Why It Is Useful

- **Core gameplay feature:** Timing is one of the most requested features in puzzle games. Players use it to challenge themselves, track personal improvement, and compare performance.
- **Engagement:** A visible timer adds a sense of urgency and increases engagement, especially for experienced players who aim for faster solves.
- **Foundation for statistics:** Solve time is a prerequisite for meaningful game statistics (see [007-game-statistics.md](007-game-statistics.md)), leaderboards, and difficulty calibration from a player perspective.
- **Competitive play:** Enables timed challenges and speed-solving modes in the future.

## Current State

- No timer or time tracking exists anywhere in the codebase.
- `gameCounterState` tracks game instance changes but not elapsed time.
- The puzzle solved state (`sudokuIsSolvedState`) is already detected, providing a natural stop trigger.

## Proposed Implementation

1. **New Jotai atoms** in `sudoku-web/src/app/state/`:
   - `timerStartState`: timestamp when the current puzzle started (reset on new game).
   - `timerElapsedState`: derived atom computing elapsed seconds from start.
   - `timerPausedState`: boolean to support pause/resume (e.g., when app is backgrounded).
   - `timerFinalState`: final solve time, set when `sudokuIsSolvedState` becomes true.

2. **Timer display component** (`Timer.tsx`):
   - Rendered in `SudokuAppBar` or `Toolbar`.
   - Shows `MM:SS` format (or `HH:MM:SS` for large puzzles).
   - Uses `requestAnimationFrame` or a 1-second `setInterval` for updates.
   - Stops and highlights the final time on puzzle completion.

3. **Lifecycle integration:**
   - Start timer on `newGame` action or puzzle import.
   - Pause timer when the browser tab is hidden (`visibilitychange` event).
   - Stop timer when `sudokuIsSolvedState` is `true`.
   - Persist the start time in `localStorage` so it survives page refreshes.

## Challenges and Risks

| Challenge | Mitigation |
|-----------|------------|
| **Performance:** Frequent re-renders from timer updates | Use `requestAnimationFrame` with a ref-based approach; only update the DOM element directly, avoiding full React re-renders. |
| **Tab visibility:** Timer continues when tab is hidden | Listen to `document.visibilitychange` and pause/resume accordingly. |
| **Undo after solve:** Player undoes the last move after timer stops | Keep the timer stopped once solved; only restart on new game. |
| **Large puzzles:** 16×16 or 25×25 puzzles may take hours | Support `HH:MM:SS` format automatically when time exceeds 60 minutes. |
| **Persistence:** Timer lost on page refresh | Persist `timerStartState` and accumulated pause duration in `localStorage`. |

## Testing Approach

- **Unit tests:**
  - Verify timer atom produces correct elapsed time given a mocked start timestamp.
  - Verify timer stops when `sudokuIsSolvedState` becomes `true`.
  - Verify timer resets when a new game is started.
  - Verify pause/resume correctly excludes paused duration from elapsed time.
- **Component tests:**
  - Render `Timer` component with mocked atoms and verify `MM:SS` format display.
  - Verify the timer display updates over time (use `vi.useFakeTimers()`).
- **Integration tests:**
  - Start a new game, verify timer is running, solve the puzzle, verify timer stops.
  - Refresh the page mid-game, verify timer resumes from persisted state.
