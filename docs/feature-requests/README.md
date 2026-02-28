# Feature Requests

This directory contains detailed feature request tickets for the Sudoku project. Each ticket documents a feature gap identified through analysis of the current codebase.

## Current Feature Summary

The application is a touch-optimized Sudoku web app built with Rust (WASM) and React/TypeScript. Current capabilities include:

- ✅ Puzzle generation with configurable difficulty (Base 2–5, multi-shot optimization)
- ✅ 9 solving strategies (Naked Singles through X-Wing and Brute Force)
- ✅ Hint system with strategy selection and auto-apply modes
- ✅ Candidate marking (pencil marks) with auto-candidate fill
- ✅ Undo/redo history
- ✅ Puzzle import/export (multiple formats, SudokuWiki sharing)
- ✅ Keyboard navigation (arrow keys, number entry, mode toggles)
- ✅ PWA with offline support and installability
- ✅ Game state persistence in localStorage
- ✅ Light and dark themes (follows OS preference)
- ✅ Error highlighting for incorrect values
- ✅ Responsive, touch-optimized UI
- ✅ Web Worker offloading for heavy computations
- ✅ World mode (experimental overlapping grids)

## Tickets

| # | Title | Priority | Dependencies |
|---|-------|----------|--------------|
| [001](001-game-timer.md) | Game Timer / Solve Time Tracking | P1 – High | None |
| [002](002-difficulty-rating-display.md) | Difficulty Rating Display in UI | P1 – High | None |
| [003](003-accessibility-improvements.md) | Accessibility (a11y) Improvements | P1 – High | None |
| [004](004-puzzle-completion-celebration.md) | Puzzle Completion Celebration | P2 – Medium | Optional: 001 |
| [005](005-manual-theme-toggle.md) | Manual Theme Toggle | P2 – Medium | None |
| [006](006-enhanced-keyboard-shortcuts.md) | Enhanced Keyboard Shortcuts | P2 – Medium | Partial: 003 |
| [007](007-game-statistics.md) | Game Statistics & History | P3 – Low | 001, 002 |
| [008](008-step-by-step-tutorial.md) | Step-by-Step Solving Tutorial | P3 – Low | 002, 003 |

## Priority Definitions

- **P1 – High:** Core gameplay or usability gaps that most users would notice. Should be addressed in the next development cycle.
- **P2 – Medium:** Quality-of-life improvements that enhance the experience. Can be addressed after P1 items.
- **P3 – Low:** Advanced features that build on P1/P2 work. Nice-to-have improvements for future iterations.

## Suggested Implementation Order

```
Phase 1 (Foundations):
  001 Game Timer ──────────────┐
  002 Difficulty Rating ───────┤
  003 Accessibility ───────────┤
                               │
Phase 2 (Polish):              │
  004 Completion Celebration ◄─┤ (optionally uses timer + difficulty)
  005 Manual Theme Toggle      │
  006 Enhanced Keyboard ◄──────┘ (related to a11y)

Phase 3 (Advanced):
  007 Game Statistics ◄──── (requires timer + difficulty)
  008 Solving Tutorial ◄─── (requires difficulty + a11y)
```

The three P1 tickets (001, 002, 003) have no dependencies and can be developed in parallel. P2 tickets have optional dependencies and can begin once their foundations are in place. P3 tickets depend on multiple P1/P2 features and should be started last.
