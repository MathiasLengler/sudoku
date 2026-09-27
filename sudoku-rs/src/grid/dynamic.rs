use anyhow::ensure;
use serde::{Deserialize, Serialize};

use crate::base::{BaseEnum, SudokuBase, match_base_enum};
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::grid::Grid;

// tsify ignores `serde(try_from, into)`.
// Default keeps the bare `DynamicGrid` in signatures valid (tsify#76).
#[cfg_attr(
    feature = "wasm",
    derive(tsify::Tsify),
    tsify(type = "T[]", type_params = "T = DynamicCell")
)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    try_from = "Vec<T>",
    into = "Vec<T>",
    rename_all = "camelCase",
    bound(
        serialize = "T: Serialize + Clone",
        deserialize = "T: Deserialize<'de>"
    )
)]
pub struct DynamicGrid<T = DynamicCell> {
    base: BaseEnum,
    cells: Vec<T>,
}

impl<T: Default + Clone> DynamicGrid<T> {
    pub fn new(base: BaseEnum) -> Self {
        Self {
            cells: match_base_enum!(base, Grid::<Base, T>::new().into_cells()),
            base,
        }
    }
}

impl<T> DynamicGrid<T> {
    pub fn base(&self) -> BaseEnum {
        self.base
    }
}

// interop `Grid<Base>`
impl<Base: SudokuBase, T: Into<DynamicCell>> TryFrom<DynamicGrid<T>> for Grid<Base> {
    type Error = Error;

    fn try_from(dynamic_grid: DynamicGrid<T>) -> Result<Self> {
        ensure!(dynamic_grid.base.is::<Base>());

        dynamic_grid.cells.try_into()
    }
}

impl<Base: SudokuBase, T, U: From<T>> From<Grid<Base, T>> for DynamicGrid<U> {
    fn from(value: Grid<Base, T>) -> Self {
        Self {
            base: Base::ENUM,
            cells: value
                .into_cells()
                .into_iter()
                .map(|cell| cell.into())
                .collect(),
        }
    }
}

impl<Base: SudokuBase, T, U: for<'a> From<&'a T>> From<&Grid<Base, T>> for DynamicGrid<U> {
    fn from(grid: &Grid<Base, T>) -> Self {
        Self {
            base: Base::ENUM,
            cells: grid.all_cells().map(|cell| cell.into()).collect(),
        }
    }
}

// interop `Vec<T>`
impl<T> From<DynamicGrid<T>> for Vec<T> {
    fn from(grid: DynamicGrid<T>) -> Self {
        grid.cells
    }
}

impl<T> TryFrom<Vec<T>> for DynamicGrid<T> {
    type Error = Error;

    fn try_from(cells: Vec<T>) -> Result<Self> {
        let base = BaseEnum::try_from_cell_count_usize(cells.len())?;

        Ok(Self { base, cells })
    }
}

// Iterators
impl DynamicGrid<DynamicCell> {
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<T> IntoIterator for DynamicGrid<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a DynamicGrid<T> {
    type Item = &'a T;

    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut DynamicGrid<T> {
    type Item = &'a mut T;

    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_from_array() {
        use serde_json::{Value, from_value, json};

        let grid: DynamicGrid = from_value(Value::Array(vec![
            json!({ "kind": "candidates", "candidates": [] });
            16
        ]))
        .unwrap();

        assert_eq!(grid.base(), BaseEnum::Base2);

        let grid: DynamicGrid = from_value(Value::Array(vec![
            json!({ "kind": "candidates", "candidates": [] });
            81
        ]))
        .unwrap();

        assert_eq!(grid.base(), BaseEnum::Base3);
    }
}
