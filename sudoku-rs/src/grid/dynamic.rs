use anyhow::ensure;
use serde::{Deserialize, Serialize};

use crate::base::{match_base_enum, BaseEnum, SudokuBase};
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::grid::Grid;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "Vec<T>", into = "Vec<T>", rename_all = "camelCase")]
pub struct DynamicGrid<T: Clone = DynamicCell> {
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

impl<T: Clone> DynamicGrid<T> {
    pub fn base(&self) -> BaseEnum {
        self.base
    }
}

// interop `Grid<Base>`
impl<Base: SudokuBase, T: Clone + Into<DynamicCell>> TryFrom<DynamicGrid<T>> for Grid<Base> {
    type Error = Error;

    fn try_from(dynamic_grid: DynamicGrid<T>) -> Result<Self> {
        ensure!(dynamic_grid.base.is::<Base>());

        dynamic_grid.cells.try_into()
    }
}

impl<Base: SudokuBase, T, U: Clone + From<T>> From<Grid<Base, T>> for DynamicGrid<U> {
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

impl<Base: SudokuBase, T, U: Clone + for<'a> From<&'a T>> From<&Grid<Base, T>> for DynamicGrid<U> {
    fn from(grid: &Grid<Base, T>) -> Self {
        Self {
            base: Base::ENUM,
            cells: grid.all_cells().map(|cell| cell.into()).collect(),
        }
    }
}

// interop `Vec<T>`
impl<T: Clone> From<DynamicGrid<T>> for Vec<T> {
    fn from(grid: DynamicGrid<T>) -> Self {
        grid.cells
    }
}

impl<T: Clone> TryFrom<Vec<T>> for DynamicGrid<T> {
    type Error = Error;

    fn try_from(cells: Vec<T>) -> Result<Self> {
        let base = BaseEnum::try_from_cell_count_usize(cells.len())?;

        Ok(Self { base, cells })
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    #![allow(clippy::all)]

    use super::*;

    // ts_rs doesn't support serde(try_from) and serde(into)
    // We use the following struct to expand the TS macro manually as a stand-in.
    // When again uncommeted, the implementation of ts_rs::TS delegates to the real DynamicGrid.

    // use ts_rs::TS;
    // #[derive(TS)]
    // #[ts(export)]
    // struct DynamicGrid<T: Clone = DynamicCell>(Vec<T>);

    // Recursive expansion of TS macro
    // ================================

    impl<T: Clone> ::ts_rs::TS for DynamicGrid<T>
    where
        T: ::ts_rs::TS,
    {
        type WithoutGenerics = DynamicGrid<::ts_rs::Dummy>;
        type OptionInnerType = Self;
        fn ident() -> String {
            ("DynamicGrid").to_string()
        }
        fn name() -> String {
            format!(
                "{}<{}>",
                "DynamicGrid",
                vec![<T as ::ts_rs::TS>::name()].join(", ")
            )
        }
        fn decl_concrete() -> String {
            format!(
                "type {} = {};",
                "DynamicGrid",
                <Self as ::ts_rs::TS>::inline()
            )
        }
        fn decl() -> String {
            #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
            struct T;

            impl std::fmt::Display for T {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl ::ts_rs::TS for T {
                type WithoutGenerics = T;
                type OptionInnerType = Self;
                fn name() -> String {
                    stringify!(T).to_owned()
                }
                fn inline() -> String {
                    panic!("{} cannot be inlined", <Self as ::ts_rs::TS>::name())
                }
                fn inline_flattened() -> String {
                    stringify!(T).to_owned()
                }
                fn decl() -> String {
                    panic!("{} cannot be declared", <Self as ::ts_rs::TS>::name())
                }
                fn decl_concrete() -> String {
                    panic!("{} cannot be declared", <Self as ::ts_rs::TS>::name())
                }
            }
            let inline = <DynamicGrid<T> as ::ts_rs::TS>::inline();
            let generics = format!(
                "<{}>",
                [format!(
                    "{} = {}",
                    "T",
                    <DynamicCell as ::ts_rs::TS>::name()
                )]
                .join(", ")
            );
            format!("type {}{generics} = {inline};", "DynamicGrid")
        }
        fn inline() -> String {
            <Vec<T> as ::ts_rs::TS>::name()
        }
        fn inline_flattened() -> String {
            panic!("{} cannot be flattened", <Self as ::ts_rs::TS>::name())
        }
        fn visit_generics(v: &mut impl ::ts_rs::TypeVisitor)
        where
            Self: 'static,
        {
            v.visit::<T>();
            <T as ::ts_rs::TS>::visit_generics(v);
        }
        fn output_path() -> Option<std::path::PathBuf> {
            Some(std::path::PathBuf::from(format!("{}.ts", "DynamicGrid")))
        }
        fn visit_dependencies(v: &mut impl ::ts_rs::TypeVisitor)
        where
            Self: 'static,
        {
            v.visit::<Vec<T>>();
            <Vec<T> as ::ts_rs::TS>::visit_generics(v);
            v.visit::<DynamicCell>();
            <DynamicCell as ::ts_rs::TS>::visit_generics(v);
        }
    }
    #[cfg(test)]
    #[test]
    fn export_bindings_dynamicgrid() {
        <DynamicGrid<::ts_rs::Dummy> as ::ts_rs::TS>::export_all().expect("could not export type");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_from_array() {
        use serde_json::{from_value, json, Value};

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
