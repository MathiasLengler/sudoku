use serde::{Deserialize, Serialize};

use super::{WorldGridDim, WorldGridPosition};

/// A tiling pattern that determines which grid positions in a world are active (contain a grid)
/// and which are gaps/holes.
///
/// Different patterns create different visual layouts and difficulty levels:
/// - Regular: All positions are active (current implementation)
/// - Chainlink: Checkerboard pattern with diagonal overlaps only
/// - Other patterns for fractal-like or hole patterns
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TilingPattern {
    /// All grid positions are active. No gaps.
    ///
    /// Visual representation (3x3 grid dim):
    /// ```text
    /// xxx
    /// xxx
    /// xxx
    /// ```
    #[default]
    Regular,

    /// Checkerboard pattern where only even positions (row + column is even) are active.
    /// Creates diagonal overlaps only (corner overlaps), increasing difficulty.
    ///
    /// Visual representation (3x3 grid dim):
    /// ```text
    /// x x
    ///  x
    /// x x
    /// ```
    ///
    /// Visual representation (5x5 grid dim):
    /// ```text
    /// x x x
    ///  x x
    /// x x x
    ///  x x
    /// x x x
    /// ```
    Chainlink,
}

impl TilingPattern {
    /// Returns all available tiling patterns.
    pub fn all() -> impl Iterator<Item = Self> {
        [Self::Regular, Self::Chainlink].into_iter()
    }

    /// Checks if a given grid position is active (should contain a grid) for this tiling pattern.
    pub fn is_position_active(self, position: WorldGridPosition) -> bool {
        match self {
            Self::Regular => true,
            Self::Chainlink => {
                // Checkerboard: position is active if (row + column) is even
                (position.row + position.column).is_multiple_of(2)
            }
        }
    }

    /// Returns an iterator over all active grid positions for the given grid dimensions.
    pub fn active_positions(
        self,
        grid_dim: WorldGridDim,
    ) -> impl Iterator<Item = WorldGridPosition> {
        grid_dim
            .all_positions()
            .filter(move |pos| self.is_position_active(*pos))
    }

    /// Returns the count of active grid positions for the given grid dimensions.
    pub fn active_position_count(self, grid_dim: WorldGridDim) -> usize {
        match self {
            Self::Regular => grid_dim.all_positions_count(),
            Self::Chainlink => {
                let total = grid_dim.all_positions_count();
                // For a checkerboard, roughly half the positions are active
                // Exact formula: ceil((rows * cols) / 2) for the "even" squares
                // When both dimensions are the same and odd: (n*n + 1) / 2
                // General case: count positions where (row + col) % 2 == 0
                let row_count = grid_dim.row_count.get();
                let col_count = grid_dim.column_count.get();

                // Number of even positions in a row×col grid:
                // If both are even: row*col / 2
                // If one is even, one is odd: row*col / 2
                // If both are odd: (row*col + 1) / 2
                if row_count % 2 == 1 && col_count % 2 == 1 {
                    total.div_ceil(2)
                } else {
                    total / 2
                }
            }
        }
    }

    /// Returns whether this pattern allows for 2D interactions between grids.
    /// Chain patterns (1D) would return false.
    pub fn has_2d_interactions(self) -> bool {
        match self {
            Self::Regular | Self::Chainlink => true,
        }
    }

    /// Returns a human-readable name for the pattern.
    pub fn name(self) -> &'static str {
        match self {
            Self::Regular => "Regular",
            Self::Chainlink => "Chainlink",
        }
    }

    /// Returns a description of the pattern.
    pub fn description(self) -> &'static str {
        match self {
            Self::Regular => "Dense grid with all positions filled and long overlap boundaries",
            Self::Chainlink => {
                "Checkerboard pattern with corner-only overlaps for increased difficulty"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_pattern() {
        let grid_dim = WorldGridDim::new(3, 3).unwrap();
        let pattern = TilingPattern::Regular;

        // All positions should be active
        let active_positions: Vec<_> = pattern.active_positions(grid_dim).collect();
        assert_eq!(active_positions.len(), 9);

        // All positions in the grid should be active
        for row in 0..3 {
            for col in 0..3 {
                assert!(pattern.is_position_active(WorldGridPosition::new(row, col)));
            }
        }

        assert_eq!(pattern.active_position_count(grid_dim), 9);
    }

    #[test]
    fn test_chainlink_pattern_3x3() {
        let grid_dim = WorldGridDim::new(3, 3).unwrap();
        let pattern = TilingPattern::Chainlink;

        // Check the expected pattern:
        // x . x
        // . x .
        // x . x
        let expected_active = [
            (0, 0),
            (0, 2),
            (1, 1),
            (2, 0),
            (2, 2),
        ];
        let expected_inactive = [
            (0, 1),
            (1, 0),
            (1, 2),
            (2, 1),
        ];

        for (row, col) in expected_active {
            assert!(
                pattern.is_position_active(WorldGridPosition::new(row, col)),
                "Position ({row}, {col}) should be active",
            );
        }

        for (row, col) in expected_inactive {
            assert!(
                !pattern.is_position_active(WorldGridPosition::new(row, col)),
                "Position ({row}, {col}) should be inactive",
            );
        }

        assert_eq!(pattern.active_position_count(grid_dim), 5);
    }

    #[test]
    fn test_chainlink_pattern_5x5() {
        let grid_dim = WorldGridDim::new(5, 5).unwrap();
        let pattern = TilingPattern::Chainlink;

        // For a 5x5 grid, positions where (row + col) is even:
        // x . x . x
        // . x . x .
        // x . x . x
        // . x . x .
        // x . x . x
        // That's 13 active positions: (5*5 + 1) / 2 = 13

        assert_eq!(pattern.active_position_count(grid_dim), 13);

        // Verify some specific positions
        assert!(pattern.is_position_active(WorldGridPosition::new(0, 0)));
        assert!(!pattern.is_position_active(WorldGridPosition::new(0, 1)));
        assert!(pattern.is_position_active(WorldGridPosition::new(2, 2)));
        assert!(!pattern.is_position_active(WorldGridPosition::new(3, 4)));
        assert!(pattern.is_position_active(WorldGridPosition::new(4, 4)));
    }

    #[test]
    fn test_chainlink_pattern_4x4() {
        let grid_dim = WorldGridDim::new(4, 4).unwrap();
        let pattern = TilingPattern::Chainlink;

        // For a 4x4 grid (even x even), exactly half are active: 16 / 2 = 8
        assert_eq!(pattern.active_position_count(grid_dim), 8);
    }

    #[test]
    fn test_chainlink_pattern_3x4() {
        let grid_dim = WorldGridDim::new(3, 4).unwrap();
        let pattern = TilingPattern::Chainlink;

        // For a 3x4 grid (odd x even), half are active: 12 / 2 = 6
        assert_eq!(pattern.active_position_count(grid_dim), 6);
    }

    #[test]
    fn test_pattern_iteration() {
        let patterns: Vec<_> = TilingPattern::all().collect();
        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&TilingPattern::Regular));
        assert!(patterns.contains(&TilingPattern::Chainlink));
    }

    #[test]
    fn test_pattern_has_2d_interactions() {
        assert!(TilingPattern::Regular.has_2d_interactions());
        assert!(TilingPattern::Chainlink.has_2d_interactions());
    }

    #[test]
    fn test_pattern_names() {
        assert_eq!(TilingPattern::Regular.name(), "Regular");
        assert_eq!(TilingPattern::Chainlink.name(), "Chainlink");
    }
}
