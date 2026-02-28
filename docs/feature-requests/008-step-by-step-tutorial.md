# 008: Step-by-Step Solving Tutorial

**Priority:** P3 – Low  
**Dependencies:** [002-difficulty-rating-display.md](002-difficulty-rating-display.md), [003-accessibility-improvements.md](003-accessibility-improvements.md)

## Summary

Add a guided tutorial mode that teaches players sudoku solving strategies step by step. The tutorial leverages the existing strategic solver and hint system to walk players through a puzzle, explaining each logical deduction in plain language with visual highlights on the grid.

## Why It Is Useful

- **Learning tool:** Many players know basic sudoku rules but struggle with intermediate and advanced strategies (Naked Pairs, X-Wing, Locked Sets). A built-in tutorial bridges this gap.
- **Leverages existing infrastructure:** The strategic solver already identifies and applies 9+ strategies. The hint system already highlights affected cells. The introspective solver was designed for step-by-step explanations. This feature builds on mature, tested code.
- **Player retention:** Players who understand more strategies find harder puzzles enjoyable rather than frustrating, increasing long-term engagement.
- **Differentiation:** Few sudoku apps provide interactive, in-context strategy explanations. This would be a standout feature.

## Current State

- **Strategic solver** implements 9 strategies with difficulty scores.
- **Hint system** (`tryStrategies()`) applies strategies and returns deductions with affected cells.
- **Hint UI** highlights cells involved in deductions (guide cells, deduction reason cells, deletion candidates).
- **Hint settings dialog** lets users select which strategies to use and configure behavior (apply mode, loop mode).
- **No tutorial mode:** There is no guided walkthrough, no plain-language explanations of strategies, and no curated progression of puzzles.

## Proposed Implementation

### Phase 1: Strategy Explanations
1. **Add explanation text to deductions** in the Rust backend:
   - Extend the `Deduction` type (or add a companion `DeductionExplanation` type) with a human-readable explanation string.
   - Example: "Cell R3C5 can only contain the value 7 because all other candidates have been eliminated (Naked Single)."
   - Example: "In Box 2, the value 4 can only go in R1C5. No other cell in the box has 4 as a candidate (Hidden Single)."
   
2. **Display explanations in the hint UI:**
   - Show the explanation text in a panel below the grid or as a popover when a hint is active.
   - Highlight the relevant cells on the grid (already supported).

### Phase 2: Guided Tutorial Mode
1. **Tutorial dialog** (`TutorialDialog.tsx`):
   - Opened from a "Learn" button in the app bar or new game dialog.
   - Presents a curated sequence of puzzles, each designed to teach a specific strategy.
   - Progression: Naked Singles → Hidden Singles → Naked Pairs → Pointing Pairs → Box/Line Reduction → X-Wing → Advanced.

2. **Curated puzzle set:**
   - Pre-generate a set of puzzles (10-15) where each puzzle requires a specific strategy to solve.
   - Store as embedded constants or bundled JSON.
   - Use the multi-shot generator with strategy-specific metrics to find ideal teaching puzzles.

3. **Step-through interface:**
   - "Next Step" button applies the next logical deduction and shows the explanation.
   - "Try It Yourself" button lets the player attempt the deduction before revealing it.
   - Progress bar shows how many steps remain.
   - Option to auto-play all steps with a configurable delay (reuse hint loop mode).

### Phase 3: Interactive Practice
1. **Practice mode** for individual strategies:
   - Generate puzzles specifically requiring a chosen strategy.
   - Highlight cells where the strategy applies and ask the player to identify the deduction.
   - Score the player's accuracy.

## Challenges and Risks

| Challenge | Mitigation |
|-----------|------------|
| **Explanation quality:** Generating clear, concise natural language explanations is hard | Start with template-based explanations (fill in row, column, cell, value, candidates). Iterate based on user feedback. Keep explanations short (1-2 sentences). |
| **Puzzle curation:** Finding puzzles that isolate a single strategy is non-trivial | Use the multi-shot generator with `StrategyApplicationCountSingle` metric to find puzzles where exactly one application of a target strategy is needed. Pre-generate and embed the puzzle set. |
| **Localization:** Explanation text needs to be translatable | Use a simple i18n approach (key-value pairs). Start with English only; extract strings for future localization. |
| **Rust-side complexity:** Adding explanation generation to the strategic solver increases code complexity | Keep explanation generation behind a feature flag or separate from the hot solving path. Only generate explanations when explicitly requested. |
| **Bundle size:** Embedded puzzle set and explanation templates add to the bundle | 15 puzzles as strings are ~2KB. Explanation templates are ~5KB. Negligible impact. |
| **Maintenance burden:** Curated puzzles may not cover all edge cases | Use automated generation rather than hand-crafting puzzles. Validate that each puzzle in the set is still solvable with the intended strategy after solver changes. |
| **Scope creep:** Tutorial mode can expand indefinitely (animations, gamification, scoring) | Define Phase 1 (explanations only) as the MVP. Phases 2 and 3 are separate iterations. |

## Testing Approach

- **Rust unit tests:**
  - Verify explanation generation produces non-empty strings for all 9 strategy types.
  - Verify explanation text includes the correct cell positions, values, and strategy name.
  - Verify explanation generation does not affect solver correctness (solving with and without explanations produces the same result).
- **Curated puzzle validation:**
  - Automated test that loads each tutorial puzzle and verifies:
    - It has a unique solution.
    - It requires the intended strategy to solve.
    - The strategic solver successfully solves it with only the strategies up to and including the intended one.
- **Frontend component tests:**
  - Render explanation panel with a mock deduction and verify the text is displayed.
  - Render tutorial dialog and verify progression through puzzles.
  - Verify "Next Step" applies a deduction and updates the grid.
  - Verify "Try It Yourself" mode accepts or rejects player input.
- **Integration tests:**
  - Start the tutorial, step through all deductions, verify the puzzle is solved at the end.
  - Verify the tutorial tracks progress and can be resumed if the page is reloaded.
- **Accessibility tests:**
  - Verify explanation text is announced by screen readers.
  - Verify tutorial navigation is keyboard-accessible.
