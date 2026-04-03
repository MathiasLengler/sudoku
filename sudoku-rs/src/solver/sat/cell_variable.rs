use crate::base::SudokuBase;
use crate::cell::Value;
use crate::error::{Error, Result};
use crate::position::{Coordinate, Position};
use anyhow::bail;
use num::Integer as _;
use std::fmt::{Display, Formatter};
use varisat::Lit;

/// A logical variable expressing that the cell at `pos` contains `value`.
///
/// Can be negated with `is_true = false`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CellVariable<Base: SudokuBase> {
    pub pos: Position<Base>,
    pub value: Value<Base>,
    pub is_true: bool,
}

impl<Base: SudokuBase> Display for CellVariable<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Self {
            pos,
            value,
            is_true,
        } = self;
        write!(f, "{pos} {} {value}", if is_true { "==" } else { "!=" })
    }
}

impl<Base: SudokuBase> From<CellVariable<Base>> for i32 {
    fn from(variable: CellVariable<Base>) -> Self {
        let i = (i32::from(variable.pos.cell_index()) * i32::from(Base::SIDE_LENGTH)
            + i32::from(Coordinate::from(variable.value).get()))
            + 1;
        if variable.is_true { i } else { -i }
    }
}

impl<Base: SudokuBase> From<CellVariable<Base>> for Lit {
    fn from(variable: CellVariable<Base>) -> Self {
        let i: i32 = variable.into();

        Lit::from_dimacs(i.try_into().unwrap())
    }
}

impl<Base: SudokuBase> TryFrom<i32> for CellVariable<Base> {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        if value == 0 {
            bail!("Zero is a invalid variable assignment")
        }

        let is_true = value.is_positive();
        let value = value.unsigned_abs() - 1;
        let (cell_index, coordinate) = value.div_rem(&Base::SIDE_LENGTH.into());

        Ok(Self {
            pos: u16::try_from(cell_index)?.try_into()?,
            value: Value::from(Coordinate::try_from(u8::try_from(coordinate)?)?),
            is_true,
        })
    }
}

impl<Base: SudokuBase> TryFrom<Lit> for CellVariable<Base> {
    type Error = Error;

    fn try_from(lit: Lit) -> Result<Self> {
        i32::try_from(lit.to_dimacs())?.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::base::consts::BaseMax;

    fn all_max_base_cell_variables() -> impl Iterator<Item = CellVariable<BaseMax>> {
        Position::<BaseMax>::all().flat_map(|pos| {
            Value::<BaseMax>::all().flat_map(move |value| {
                [true, false].iter().map(move |&is_true| CellVariable {
                    pos,
                    value,
                    is_true,
                })
            })
        })
    }

    #[test]
    fn test_i32_roundtrip() {
        for variable in all_max_base_cell_variables() {
            assert_eq!(variable, i32::from(variable).try_into().unwrap());
        }
    }

    #[test]
    fn test_i32_unique() {
        use itertools::Itertools;

        assert!(all_max_base_cell_variables().all_unique());
    }
}
