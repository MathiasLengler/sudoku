# TODOs

## UI
- guide highlighting
  - [x] value
    - [ ] with/without their groups
    - candidates?
- mark conflicting cells

## Tooling
- PWA
  - offline
  - add to home
  - fullscreen
  - Framework?
- azure pipelines
  - run linting
    - eslint
    - clippy
- evaluate useReducer
- criterion benchmark
  - [ ] sudoku
    - [X] has_conflict_at
    - [X] has_duplicate
    - [X] all_positions
    - [ ] update_candidates
    - [X] direct_candidates
    - [ ] set_all_direct_candidates
## UX
- save (local storage)
  - settings
  - current board
  - highscore

### Selector Control Panel:
- [ ] sticky value toggle
  - no selected cell when active
  - guide for all values equal to sticky value
- [ ] undo/redo buttons
  - sudokuCan{Undo|Redo}
### New sudoku:
- Generate with difficulty
  - by number of empty cells
  - by needed strategies
- Editable mode
  - freeze button
  - use cases:
    - input for solver
    - interactive import
- Import
  - Does a sudoku exchange format exist?
    - String of values
      - Empty cell as `[\.\-0]`
      - optional spaces between values
      - optional newline between rows
### Gameplay Options
- fill direct candidates on new sudoku
- remove direct candidates on set value (is implemented and hardcoded true)
- highlight incorrect value
  - Sudoku needs a solved grid
  - add flag to TransportSudoku
### Solver Controls
- Solve button?
- Solver selector
  - Backtracking
  - Strategic
  - Hybrid
- Options
    - animation / speed
    - select strategies
