# TODOs

## UI
- guide highlighting
  - [X] selected cell
  - [X] row
  - [X] column
  - [X] block
  - [ ] value
- mark conflicting cells
- responsive aspect ratio
  - control placement
  - grid size respects control size
  - root sudoku has viewport with/height
    - handle overflow/force grid to size?

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
- Hotkey for candidate mode toggle
- save settings

### Selector Control Panel:
- [X] set candidate mode toggle
- [ ] sticky value toggle
- [X] delete cell button
- [ ] undo button
- [ ] redo button
### Gameplay Options
- fill direct candidates (on start vs button)
- remove direct candidates on set value
- highlight incorrect value
### Solver Controls
- Solve button
- Solver selector
- Options
  - Backtracking
    - animation
  - Strategic
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
