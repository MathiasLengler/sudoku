use std::fmt::{Display, Formatter};

use anyhow::bail;
use num::Integer as _;
use varisat::Lit;

use crate::{
    base::SudokuBase,
    cell::Value,
    error::{Error, Result},
    position::Coordinate,
};

// TODO: abstract `CellVariable` and `GroupCellVariable`

/// A logical variable expressing that the cell inside a group at `coordinate` contains a `candidate`.
///
/// Can be negated with `is_true = false`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(super) struct GroupCellVariable<Base: SudokuBase> {
    // The "index" of the cell inside the group
    coordinate: Coordinate<Base>,
    // The specific candidate this variable represents
    candidate: Value<Base>,
    // Whether the cell contains the candidate or not
    is_true: bool,
}

impl<Base: SudokuBase> Display for GroupCellVariable<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Self {
            coordinate,
            candidate,
            is_true,
        } = self;
        write!(
            f,
            "{coordinate} {} {candidate}",
            if is_true { "==" } else { "!=" }
        )
    }
}

impl<Base: SudokuBase> From<GroupCellVariable<Base>> for i32 {
    fn from(variable: GroupCellVariable<Base>) -> Self {
        let i = (i32::from(variable.coordinate.get()) * i32::from(Base::SIDE_LENGTH)
            + i32::from(Coordinate::from(variable.candidate).get()))
            + 1;
        if variable.is_true {
            i
        } else {
            -i
        }
    }
}

impl<Base: SudokuBase> From<GroupCellVariable<Base>> for Lit {
    fn from(variable: GroupCellVariable<Base>) -> Self {
        let i: i32 = variable.into();

        Lit::from_dimacs(i.try_into().unwrap())
    }
}

impl<Base: SudokuBase> TryFrom<i32> for GroupCellVariable<Base> {
    type Error = Error;

    fn try_from(i: i32) -> Result<Self> {
        if i == 0 {
            bail!("Zero is a invalid variable assignment")
        }

        let is_true = i.is_positive();
        let value = i.unsigned_abs() - 1;
        let (coordinate, candidate_as_coordinate) = value.div_rem(&Base::SIDE_LENGTH.into());

        Ok(Self {
            coordinate: u8::try_from(coordinate)?.try_into()?,
            candidate: Value::from(Coordinate::try_from(u8::try_from(
                candidate_as_coordinate,
            )?)?),
            is_true,
        })
    }
}

impl<Base: SudokuBase> TryFrom<Lit> for GroupCellVariable<Base> {
    type Error = Error;

    fn try_from(lit: Lit) -> Result<Self> {
        i32::try_from(lit.to_dimacs())?.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::base::consts::BaseMax;

    fn all_max_base_group_cell_variables() -> impl Iterator<Item = GroupCellVariable<BaseMax>> {
        Coordinate::<BaseMax>::all().flat_map(|coordinate| {
            Value::<BaseMax>::all().flat_map(move |candidate| {
                [true, false].iter().map(move |&is_true| GroupCellVariable {
                    coordinate,
                    candidate,
                    is_true,
                })
            })
        })
    }

    #[test]
    fn test_i32_roundtrip() {
        for variable in all_max_base_group_cell_variables() {
            assert_eq!(variable, i32::from(variable).try_into().unwrap());
        }
    }

    #[test]
    fn test_i32_unique() {
        use itertools::Itertools;

        assert!(all_max_base_group_cell_variables().all_unique());
    }
}
