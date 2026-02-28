pub use brute_force::BruteForce;
pub use group_intersection::{
    GroupIntersectionAxisToBlock, GroupIntersectionBlockToAxis, GroupIntersectionBoth,
};
pub use hidden_singles::HiddenSingles;
pub use locked_sets::LockedSets;
pub use naked_pairs::NakedPairs;
pub use naked_singles::NakedSingles;
pub use unique_rectangles::UniqueRectangles;
pub use x_wing::XWing;

// Strategies
mod brute_force;
mod group_intersection;
mod hidden_singles;
pub mod locked_sets;
mod naked_pairs;
mod naked_singles;
mod unique_rectangles;
mod x_wing;
