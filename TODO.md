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

## UX
- save (local storage)
  - settings
  - current board
  - highscore

### Selector Control Panel:
- [ ] sticky value toggle
  - no selected cell when active
  - guide for all values equal to sticky value
- [ ] undo button
  - sudokuCanUndo
- [ ] redo button
  - sudokuCanRedo
### Gameplay Options
- fill direct candidates on new sudoku
- remove direct candidates on set value (is implemented and hardcoded true)
- highlight incorrect value
### Solver Controls
- Solve button
- Solver selector
  - Backtracking
  - Strategic
- Options
    - animation / speed
    - select strategies
### New sudoku:
- Generate with difficulty
  - by number of empty cells
  - by needed strategies
- Editable
  - freeze button
  - input for solver
- Import
  - Does a sudoku exchange format exist?
