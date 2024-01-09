pub use compact::*;

mod compact;
pub mod dynamic;

// TODO: Cell without fixed state ("SimpleCell")
//  could be useful for:
//  - strategic::Solver operating on a grid of SimpleCells
//    - for solvers, every value is fixed
//  - more memory-compact 2d array, with fixed state in separate bitfield
//  - even more compact Grid<Base, Option<Value<Base>>> for backtracking::Solver
