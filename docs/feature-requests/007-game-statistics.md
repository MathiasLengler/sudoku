# 007: Game Statistics & History

**Priority:** P3 – Low  
**Dependencies:** [001-game-timer.md](001-game-timer.md), [002-difficulty-rating-display.md](002-difficulty-rating-display.md)

## Summary

Track and display player statistics including games played, games completed, average solve times, best times per difficulty, win streaks, and a history of recent games. All data is stored locally in the browser using `localStorage` or `IndexedDB`.

## Why It Is Useful

- **Progress tracking:** Players want to see their improvement over time. Statistics provide measurable evidence of skill growth.
- **Motivation and retention:** Streaks, personal bests, and completion rates are proven engagement drivers in puzzle and casual games.
- **Gameplay context:** Knowing "I've solved 50 medium puzzles with an average time of 8:32" gives players a sense of accomplishment beyond the current puzzle.
- **No backend required:** All statistics can be computed and stored client-side, keeping the app fully offline-capable.

## Current State

- No statistics tracking exists.
- `gameCounterState` increments on new games but the count is not persisted or displayed.
- Solve time is not tracked (depends on [001-game-timer.md](001-game-timer.md)).
- Difficulty rating is not surfaced (depends on [002-difficulty-rating-display.md](002-difficulty-rating-display.md)).
- `localStorage` persistence infrastructure exists (`localStorageEffect.ts`, `getZodLocalStorage`).

## Proposed Implementation

1. **Statistics data model** (`GameRecord` type):
   ```typescript
   type GameRecord = {
     id: string;               // UUID
     date: string;             // ISO 8601 timestamp
     base: number;             // Grid base size (3 = 9×9)
     difficulty: string;       // "Easy" | "Medium" | "Hard" | "Expert"
     strategyScore: number;    // Raw difficulty score
     givensCount: number;      // Number of clues
     solveTimeSeconds: number; // Time to solve
     completed: boolean;       // Whether the puzzle was solved
     hintsUsed: number;        // Number of hints requested
   };
   ```

2. **Statistics aggregation** (`GameStatistics` type):
   ```typescript
   type GameStatistics = {
     totalGamesStarted: number;
     totalGamesCompleted: number;
     completionRate: number;           // percentage
     currentStreak: number;            // consecutive completions
     bestStreak: number;
     averageSolveTime: number;         // seconds
     bestSolveTime: number;            // seconds
     byDifficulty: Record<string, {
       played: number;
       completed: number;
       averageTime: number;
       bestTime: number;
     }>;
   };
   ```

3. **Storage layer:**
   - Use `IndexedDB` (via `idb-keyval` or similar lightweight wrapper) for game history (can grow large).
   - Use `localStorage` for aggregated statistics (small, fast access).
   - Limit history to the last 500 games to prevent unbounded growth.

4. **Statistics page/dialog** (`StatisticsDialog.tsx`):
   - Accessible from the app bar via a chart/stats icon button.
   - Shows summary cards: games played, completion rate, current streak, best time.
   - Shows a breakdown table by difficulty.
   - Optionally shows a simple chart of solve times over the last 30 games (using CSS-only bar chart or a minimal chart library).

5. **Record lifecycle:**
   - Create a `GameRecord` when a new game starts.
   - Update `solveTimeSeconds`, `hintsUsed`, and `completed` during gameplay.
   - Finalize the record when the puzzle is solved or a new game is started.

## Challenges and Risks

| Challenge | Mitigation |
|-----------|------------|
| **Storage limits:** `localStorage` has a 5-10MB limit; `IndexedDB` is more flexible | Use `IndexedDB` for game history. Cap history at 500 records. |
| **Data migration:** Schema changes in future versions may break stored data | Version the data schema (e.g., `stats_v1`). Write migration logic for schema changes. Use Zod for validation on load. |
| **Dependency chain:** Full statistics require timer (001) and difficulty (002) | Implement statistics incrementally: track games played/completed first (no dependencies), add time and difficulty when available. |
| **Performance:** Aggregating 500 records on page load | Pre-compute aggregated statistics and store them separately. Only recompute when a new record is added. |
| **Privacy concerns:** Tracking gameplay data may feel invasive | All data is local-only and never transmitted. Document this clearly. Provide a "Clear Statistics" button. |
| **Bundle size:** Chart libraries can be large | Use CSS-only charts (flexbox bars) or a minimal library like `uplot` (~8KB). Avoid heavy libraries like Chart.js (~65KB min). |

## Testing Approach

- **Unit tests:**
  - Create mock `GameRecord` entries and verify `GameStatistics` aggregation is correct.
  - Verify streak calculation: consecutive completions increment streak; an incomplete game resets it.
  - Verify averages, bests, and completion rates with known data.
  - Verify history cap: adding the 501st record removes the oldest.
- **Storage tests:**
  - Verify records persist to `IndexedDB` and load correctly.
  - Verify aggregated stats persist to `localStorage`.
  - Verify data migration from `stats_v1` to `stats_v2` (when needed).
  - Verify graceful handling of corrupted data (Zod validation).
- **Component tests:**
  - Render `StatisticsDialog` with mocked data and verify all summary cards display correctly.
  - Verify difficulty breakdown table shows correct values.
  - Verify "Clear Statistics" button resets all data.
- **Integration tests:**
  - Play and complete a game, verify a record is created with correct data.
  - Play multiple games, verify statistics update incrementally.
  - Verify statistics persist across page reloads.
