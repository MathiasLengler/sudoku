use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::strategic::strategies::*;
use anyhow::{bail, ensure, Context};
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::btree_map::Values;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Deductions<Base: SudokuBase> {
    // Invariant: K == V.pos
    deductions: BTreeMap<Position, Deduction<Base>>,
}

/// Specialization workaround.
///
/// Reference: https://github.com/rust-lang/rust/issues/50133#issuecomment-646908391
#[derive(Debug)]
pub struct IntoDeductions<I>(pub I);

impl<Base: SudokuBase, I: IntoIterator<Item = Deduction<Base>>> TryFrom<IntoDeductions<I>>
    for Deductions<Base>
{
    type Error = Error;

    fn try_from(into_deductions: IntoDeductions<I>) -> Result<Self> {
        let deductions = into_deductions.0;

        let mut this = Self::default();

        for deduction in deductions {
            this.try_append(deduction)?;
        }
        Ok(this)
    }
}

/// Specialization workaround.
///
/// Reference: https://github.com/rust-lang/rust/issues/50133#issuecomment-646908391
#[derive(Debug)]
pub struct TryIntoDeductions<I>(pub I);

impl<Base: SudokuBase, I: IntoIterator<Item = Result<Deduction<Base>>>>
    TryFrom<TryIntoDeductions<I>> for Deductions<Base>
{
    type Error = Error;

    fn try_from(into_deduction_results: TryIntoDeductions<I>) -> Result<Self> {
        let deduction_results = into_deduction_results.0;

        let mut this = Self::default();

        for deduction in deduction_results {
            this.try_append(deduction?)?
        }
        Ok(this)
    }
}

impl<Base: SudokuBase> IntoIterator for Deductions<Base> {
    type Item = Deduction<Base>;
    type IntoIter = std::collections::btree_map::IntoValues<Position, Deduction<Base>>;

    fn into_iter(self) -> Self::IntoIter {
        self.deductions.into_values()
    }
}

impl<'a, Base: SudokuBase> IntoIterator for &'a Deductions<Base> {
    type Item = &'a Deduction<Base>;
    type IntoIter = Values<'a, Position, Deduction<Base>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Base: SudokuBase> Deductions<Base> {
    pub fn iter(&self) -> Values<'_, Position, Deduction<Base>> {
        self.deductions.values()
    }

    pub fn is_empty(&self) -> bool {
        self.deductions.is_empty()
    }

    fn try_append(&mut self, deduction: Deduction<Base>) -> Result<()> {
        if let Some(existing_deduction) = self.deductions.get(&deduction.pos) {
            self.deductions
                .insert(deduction.pos, existing_deduction.merge(&deduction)?);
        } else {
            self.deductions.insert(deduction.pos, deduction);
        }

        Ok(())
    }

    pub fn apply(&self, grid: &mut Grid<Base>) {
        for deduction in self {
            deduction.apply(grid);
        }

        for deduction in self {
            grid.direct_candidates(deduction.pos);
        }
    }
}

// TODO: &grid.deduction_at
// TODO: test order
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Deduction<Base: SudokuBase> {
    pos: Position,
    kind: DeductionKind<Base>,
    previous_candidates: Candidates<Base>,
}

impl<Base: SudokuBase> Display for Deduction<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Deduction {
            pos,
            previous_candidates,
            kind,
        } = self;

        match kind {
            DeductionKind::Value { value } => {
                write!(f, "At position {pos} previous candidates {previous_candidates} must be the value {value}")
            }
            DeductionKind::PruneCandidates {
                remaining_candidates,
            } => {
                write!(f, "At position {pos} previous candidates {previous_candidates} must be {remaining_candidates}")
            }
        }
    }
}

impl<Base: SudokuBase> Deduction<Base> {
    fn new(
        pos: Position,
        previous_candidates: Candidates<Base>,
        kind: DeductionKind<Base>,
    ) -> Result<Self> {
        let this = Self {
            pos,
            previous_candidates,
            kind,
        };

        // Invariant: only validated Deductions must be returned.
        this.validate()?;

        Ok(this)
    }

    pub fn with_value(
        pos: Position,
        previous_candidates: Candidates<Base>,
        value: Value<Base>,
    ) -> Result<Self> {
        Self::new(pos, previous_candidates, DeductionKind::with_value(value))
    }

    pub fn with_remaining_candidates(
        pos: Position,
        previous_candidates: Candidates<Base>,
        remaining_candidates: Candidates<Base>,
    ) -> Result<Self> {
        Self::new(
            pos,
            previous_candidates,
            DeductionKind::with_remaining_candidates(remaining_candidates)?,
        )
    }

    pub fn apply(&self, grid: &mut Grid<Base>) {
        let Deduction {
            pos,
            previous_candidates,
            kind,
        } = *self;

        let cell = grid.get_mut(pos);
        debug_assert_eq!(cell.candidates(), Some(previous_candidates));
        match kind {
            DeductionKind::Value { value } => {
                cell.set_value(value);
            }
            DeductionKind::PruneCandidates {
                remaining_candidates,
            } => {
                cell.set_candidates(remaining_candidates);
            }
        }
    }

    fn validate(&self) -> Result<()> {
        let Deduction {
            previous_candidates,
            kind,
            ..
        } = self;

        ensure!(
            !previous_candidates.is_empty(),
            "Unexpected deduction for previously empty candidates: {self}"
        );

        match kind {
            DeductionKind::Value { value } => {
                ensure!(
                    previous_candidates.has(*value),
                    "Unexpected value deduction not in previous candidates: {self}"
                );
            }
            DeductionKind::PruneCandidates {
                remaining_candidates,
            } => {
                ensure!(
                    previous_candidates != remaining_candidates,
                    "Unexpected no-op deduction: {self}"
                );

                let added_candidates = remaining_candidates.without(&previous_candidates);
                ensure!(
                    added_candidates.is_empty(),
                    "Unexpected candidate(s) addition: {self}"
                )
            }
        }

        Ok(())
    }

    fn merge(&self, other: &Self) -> Result<Self> {
        if self == other {
            return Ok(*self);
        }

        let Deduction {
            pos: self_pos,
            previous_candidates: self_previous_candidates,
            kind: self_kind,
        } = self;

        let Deduction {
            pos: other_pos,
            previous_candidates: other_previous_candidates,
            kind: other_kind,
        } = other;

        // "Try block"
        (|| {
            ensure!(
                self_pos == other_pos,
                "Conflicting positions: {self_pos} != {other_pos}"
            );

            ensure!(
                self_previous_candidates == other_previous_candidates,
                "Conflicting previous_candidates: {self_previous_candidates} != {other_previous_candidates}"
            );

            Self::new(*self_pos, *self_previous_candidates, self_kind.merge(&other_kind)?)
        })().with_context(|| format!("Incompatible merge of two deductions: {self}, {other}"))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum DeductionKind<Base: SudokuBase> {
    Value {
        value: Value<Base>,
    },
    PruneCandidates {
        remaining_candidates: Candidates<Base>,
    },
}

impl<Base: SudokuBase> DeductionKind<Base> {
    fn with_value(value: Value<Base>) -> Self {
        DeductionKind::Value { value }
    }

    fn with_remaining_candidates(remaining_candidates: Candidates<Base>) -> Result<Self> {
        ensure!(
            !remaining_candidates.is_empty(),
            "At least one candidate must be remaining"
        );

        Ok(DeductionKind::PruneCandidates {
            remaining_candidates,
        })
    }

    fn merge(&self, other: &Self) -> Result<Self> {
        use DeductionKind::*;
        Ok(match (*self, *other) {
            (Value { value: self_value }, Value { value: other_value }) => {
                bail!("Conflicting values: {self_value}, {other_value}")
            }
            // Merge PruneCandidates by intersecting their candidates.
            (
                PruneCandidates {
                    remaining_candidates: self_remaining_candidates,
                    ..
                },
                PruneCandidates {
                    remaining_candidates: other_remaining_candidates,
                    ..
                },
            ) => DeductionKind::with_remaining_candidates(
                self_remaining_candidates.intersection(&other_remaining_candidates),
            )?,
            // More specific Value overwrites PruneCandidates
            (Value { value }, _) | (_, Value { value }) => DeductionKind::with_value(value),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;
    use crate::cell::Cell;
    use crate::samples;

    #[test]
    fn test_deduction_apply() {
        let mut grid = samples::base_2_candidates_coordinates();

        let pos = Position { row: 0, column: 1 };
        let value = 1.try_into().unwrap();
        Deduction::with_value(pos, Candidates::single(1.try_into().unwrap()), value)
            .unwrap()
            .apply(&mut grid);
        assert_eq!(*grid.get(pos), Cell::with_value(value, false));

        let pos = Position { row: 3, column: 3 };
        let candidates = vec![2, 4].try_into().unwrap();
        Deduction::with_remaining_candidates(pos, Candidates::all(), candidates)
            .unwrap()
            .apply(&mut grid);
        assert_eq!(*grid.get(pos), Cell::with_candidates(candidates));
    }

    #[test]
    fn test_deduction_merge() {
        let pos = Position { row: 1, column: 1 };
        let previous_candidates: Candidates<U2> = Candidates::all();
        let remaining_candidates: Candidates<U2> = Candidates::single(1.try_into().unwrap());
        let value: Value<U2> = 1.try_into().unwrap();

        let cases: Vec<(Deduction<U2>, Deduction<U2>, Deduction<U2>)> = vec![
            // Equal
            (
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
            ),
            // Left Value overwrites right PruneCandidates
            (
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    remaining_candidates,
                )
                .unwrap(),
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
            ),
            // Right Value overwrites left PruneCandidates
            (
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    remaining_candidates,
                )
                .unwrap(),
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
            ),
            // Intersect PruneCandidates
            (
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    vec![1, 2, 4].try_into().unwrap(),
                )
                .unwrap(),
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    vec![1, 3, 4].try_into().unwrap(),
                )
                .unwrap(),
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    vec![1, 4].try_into().unwrap(),
                )
                .unwrap(),
            ),
        ];

        for (left_strategy, right_strategy, expected_strategy) in cases {
            assert_eq!(
                left_strategy.merge(&right_strategy).unwrap(),
                expected_strategy
            );
        }
    }
    #[test]
    fn test_deduction_merge_err() {
        let pos = Position { row: 1, column: 1 };
        let different_pos = Position { row: 2, column: 2 };
        let previous_candidates: Candidates<U2> = Candidates::all();
        let different_previous_candidates: Candidates<U2> = vec![1, 2].try_into().unwrap();
        let remaining_candidates: Candidates<U2> = Candidates::single(1.try_into().unwrap());
        let different_remaining_candidates: Candidates<U2> =
            Candidates::single(2.try_into().unwrap());
        let value: Value<U2> = 1.try_into().unwrap();
        let different_value: Value<U2> = 2.try_into().unwrap();

        let err_cases: Vec<(Deduction<U2>, Deduction<U2>)> = vec![
            // Different pos
            (
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
                Deduction::with_value(different_pos, previous_candidates, value).unwrap(),
            ),
            (
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    remaining_candidates,
                )
                .unwrap(),
                Deduction::with_remaining_candidates(
                    different_pos,
                    previous_candidates,
                    remaining_candidates,
                )
                .unwrap(),
            ),
            // Different previous_candidates
            (
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
                Deduction::with_value(pos, different_previous_candidates, value).unwrap(),
            ),
            (
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    remaining_candidates,
                )
                .unwrap(),
                Deduction::with_remaining_candidates(
                    pos,
                    different_previous_candidates,
                    remaining_candidates,
                )
                .unwrap(),
            ),
            // Different value
            (
                Deduction::with_value(pos, previous_candidates, value).unwrap(),
                Deduction::with_value(pos, previous_candidates, different_value).unwrap(),
            ),
            // No intersection
            (
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    remaining_candidates,
                )
                .unwrap(),
                Deduction::with_remaining_candidates(
                    pos,
                    previous_candidates,
                    different_remaining_candidates,
                )
                .unwrap(),
            ),
        ];

        for (left_strategy, right_strategy) in err_cases {
            assert!(left_strategy.merge(&right_strategy).is_err());
        }
    }
}
