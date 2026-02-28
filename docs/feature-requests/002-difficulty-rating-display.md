# 002: Difficulty Rating Display in UI

**Priority:** P1 – High  
**Dependencies:** None (backend metrics already exist)

## Summary

Surface the puzzle difficulty rating to the player in the UI after puzzle generation. The Rust backend already computes extensive grid metrics (strategy score, backtrack count, clue count, etc.) but none of this information is currently displayed to the user.

## Why It Is Useful

- **Informed gameplay:** Players want to know how hard the puzzle they are about to solve is. Showing a difficulty label (e.g., "Easy", "Medium", "Hard", "Expert") sets expectations and helps players select appropriate challenges.
- **Skill progression:** Displaying difficulty helps players gradually increase challenge as their skills improve.
- **Leverages existing infrastructure:** The `GridMetric` enum in `sudoku-rs` already computes 11+ metrics including `StrategyScore`, `BacktrackCount`, `GridGivensCount`, and more. The multi-shot generator already evaluates these metrics — they just need to be surfaced.
- **Differentiation:** Most sudoku apps show difficulty levels; not showing one is a notable gap.

## Current State

- `GridMetric` enum in `sudoku-rs/src/generator/` computes metrics like:
  - `StrategyScore` – aggregate difficulty score based on strategies needed
  - `BacktrackCount` – number of backtracks needed to solve
  - `GridGivensCount` – number of given clues
  - `StrategyApplicationCountAny` – how many strategy applications needed
- Multi-shot generator already evaluates and optimizes for these metrics.
- The `TransportSudoku` type (sent to the frontend) does not include metric data.
- The UI shows no difficulty information after generation.

## Proposed Implementation

1. **Extend `TransportSudoku`** (or add a companion type) to include an optional `DifficultyInfo` struct:
   ```rust
   #[derive(Serialize, ts_rs::TS)]
   pub struct DifficultyInfo {
       pub strategy_score: f64,
       pub label: DifficultyLabel, // Easy, Medium, Hard, Expert
       pub givens_count: usize,
       pub strategies_required: Vec<String>,
   }
   ```

2. **Compute difficulty on generation:** After generating a puzzle, run the strategy solver to determine which strategies are required and compute the strategy score. Map the score to a human-readable label using configurable thresholds.

3. **Add WASM API:** Expose `getDifficultyInfo()` on `WasmSudoku` to return difficulty data for the current puzzle.

4. **Frontend display:**
   - Show a difficulty badge/chip in the `SudokuAppBar` (e.g., "Medium 🟡").
   - Optionally show a tooltip with detailed metrics (strategy score, strategies used, givens count).
   - Use MUI `Chip` or `Badge` component with color coding (green/yellow/orange/red).

5. **Difficulty for imported puzzles:** When a puzzle is imported, analyze it on-demand and display the computed difficulty.

## Challenges and Risks

| Challenge | Mitigation |
|-----------|------------|
| **Score calibration:** Mapping raw strategy scores to labels is subjective | Start with simple thresholds based on the `StrategyScore` metric. Iterate based on player feedback. Document the mapping clearly. |
| **Performance for imports:** Analyzing imported puzzles adds latency | Run difficulty analysis in the web worker asynchronously. Show a loading indicator while computing. |
| **Large grids:** Difficulty metrics may not be comparable across base sizes (9×9 vs 16×16) | Normalize scores per base size or only show labels for Base3 (standard 9×9) initially. |
| **Metric stability:** Different metrics may disagree on difficulty | Use `StrategyScore` as the primary metric; show others as supplementary detail in a tooltip. |
| **Type generation:** New Rust types need TypeScript bindings | Use existing `ts-rs` infrastructure; run `just generate-tsrs-bindings` after adding the new struct. |

## Testing Approach

- **Rust unit tests:**
  - Generate puzzles of known difficulty and verify the computed `DifficultyLabel` matches expectations.
  - Verify that `DifficultyInfo` serialization produces valid JSON.
  - Test threshold boundaries (e.g., scores at the exact boundary between "Medium" and "Hard").
- **WASM integration tests:**
  - Generate a puzzle via WASM, call `getDifficultyInfo()`, and verify the returned object has the expected shape.
- **Frontend component tests:**
  - Render the difficulty badge with mocked `DifficultyInfo` and verify the correct label and color are displayed.
  - Verify tooltip shows detailed metrics on hover.
- **End-to-end tests:**
  - Generate a new game, verify difficulty badge appears.
  - Import a puzzle, verify difficulty is computed and displayed.
