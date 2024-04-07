use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelativeDir {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl RelativeDir {
    pub fn all() -> impl Iterator<Item = Self> {
        use RelativeDir::*;

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

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Quadrant {
    pub fn to_relative_dir(self) -> RelativeDir {
        use Quadrant::*;

        match self {
            TopLeft => RelativeDir::TopLeft,
            TopRight => RelativeDir::TopRight,
            BottomLeft => RelativeDir::BottomLeft,
            BottomRight => RelativeDir::BottomRight,
        }
    }

    pub fn all() -> impl Iterator<Item = Self> {
        use Quadrant::*;

        [TopLeft, TopRight, BottomLeft, BottomRight].into_iter()
    }
}
