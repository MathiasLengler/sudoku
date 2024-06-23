pub use backtracking::Backtracking;
pub use group_intersection::{
    GroupIntersectionAxisToBlock, GroupIntersectionBlockToAxis, GroupIntersectionBoth,
};
pub use group_reduction::GroupReduction;
pub use hidden_singles::HiddenSingles;
pub use naked_pairs::NakedPairs;
pub use naked_singles::NakedSingles;

// Strategies
mod backtracking;
mod group_intersection;
pub mod group_reduction;
mod hidden_singles;
mod naked_pairs;
mod naked_singles;
