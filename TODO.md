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
- Edit Mode

### Solver Controls
- Solve all
- animation / speed
- select strategies until stuck

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
