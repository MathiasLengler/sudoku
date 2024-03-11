use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TileIndex {
    pub row: usize,
    pub column: usize,
}

impl TileIndex {
    pub fn contained_in(self, tile_dim: TileDim) -> bool {
        tile_dim.contains(self)
    }

    pub fn adjacent(self, dir: RelativeTileDir, tile_dim: TileDim) -> Option<Self> {
        let TileIndex { row, column } = self;

        let adjacent = match dir {
            RelativeTileDir::TopLeft => Self {
                row: row.checked_sub(1)?,
                column: column.checked_sub(1)?,
            },
            RelativeTileDir::Left => Self {
                row,
                column: column.checked_sub(1)?,
            },
            RelativeTileDir::Right => Self {
                row,
                column: column + 1,
            },
            RelativeTileDir::TopRight => Self {
                row: row.checked_sub(1)?,
                column: column + 1,
            },
            RelativeTileDir::Top => Self {
                row: row.checked_sub(1)?,
                column,
            },
            RelativeTileDir::BottomLeft => Self {
                row: row + 1,
                column: column.checked_sub(1)?,
            },

            RelativeTileDir::Bottom => Self {
                row: row + 1,
                column,
            },
            RelativeTileDir::BottomRight => Self {
                row: row + 1,
                column: column + 1,
            },
        };

        if adjacent.contained_in(tile_dim) {
            Some(adjacent)
        } else {
            None
        }
    }

    pub fn is_at_top_edge(self) -> bool {
        self.row == 0
    }

    pub fn is_at_left_edge(self) -> bool {
        self.column == 0
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TileDim {
    pub row_count: usize,
    pub column_count: usize,
}

impl TileDim {
    pub fn contains(self, index: TileIndex) -> bool {
        let TileDim {
            row_count,
            column_count,
        } = self;
        let TileIndex { row, column } = index;

        (0..row_count).contains(&row) && (0..column_count).contains(&column)
    }

    pub fn all_indexes(self) -> impl Iterator<Item = TileIndex> {
        (0..self.row_count).flat_map(move |row| {
            (0..self.column_count).map(move |column| TileIndex { row, column })
        })
    }

    pub fn tile_count(self) -> usize {
        let TileDim {
            row_count,
            column_count,
        } = self;

        row_count * column_count
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelativeTileDir {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl RelativeTileDir {
    pub fn all() -> impl Iterator<Item = Self> {
        use RelativeTileDir::*;

        [
            TopLeft,
            Top,
            TopRight,
            Left,
            Right,
            BottomLeft,
            Bottom,
            BottomRight,
        ]
        .into_iter()
    }
}
