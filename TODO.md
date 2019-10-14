# TODOs

## UI
- guide highlighting
  - [x] value
    - [x] with/without their groups
    - [ ] candidates
- mark conflicting cells

## UX
- save (local storage)
  - settings
  - current board
  - highscore
- Share Sudoku
  - export as string
  - provide link / sync with url search param

### Selector Control Panel:
- [ ] undo/redo buttons
  - sudokuCan{Undo|Redo}

### New sudoku:
- Generate with difficulty
  - by needed strategies
- Editable mode
  - freeze button
  - use cases:
    - input for solver
    - interactive import

### Gameplay Options
- fill direct candidates on new sudoku
- remove direct candidates on set value (is implemented and hardcoded true)
- highlight incorrect value
  - Sudoku needs a solved grid
  - add flag to TransportSudoku
  - Model State Diagram of Sudoku
    - Edit Mode
    - Play Mode

### Solver Controls
- Solve button?
- Solver selector
  - Backtracking
  - Strategic
  - Hybrid
- Options
    - animation / speed
    - select strategies

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
