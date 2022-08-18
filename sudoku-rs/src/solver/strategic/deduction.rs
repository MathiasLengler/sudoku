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

// TODO: introduce wrapper struct
//  top-level fields:
//  - pos
//  - previous_candidates
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OldDeduction<Base: SudokuBase> {
    Value {
        pos: Position,
        value: Value<Base>,
    },
    PruneCandidates {
        pos: Position,
        previous_candidates: Candidates<Base>,
        remaining_candidates: Candidates<Base>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Deductions<Base: SudokuBase> {
    // Invariant: K == V.pos
    deductions: BTreeMap<Position, Deduction<Base>>,
}

/// Specification workaround.
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

/// Specification workaround.
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
                // FIXME: this breaks the debug assert above (grid.apply_deductions)
                grid.update_candidates(pos, value);
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

            Ok(Self {
                pos: *self_pos,
                previous_candidates: *self_previous_candidates,
                kind: self_kind.merge(&other_kind)?,
            })

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

impl<Base: SudokuBase> Ord for OldDeduction<Base> {
    fn cmp(&self, other: &Self) -> Ordering {
        use std::mem::discriminant;

        use OldDeduction::*;

        // First, sort by Position
        self.position()
            .cmp(&other.position())
            // Then discriminant
            .then_with(|| self.discriminant_order().cmp(&other.discriminant_order()))
            // Then properties
            .then_with(|| match (self, other) {
                (
                    Value {
                        pos: self_pos,
                        value: self_value,
                    },
                    Value {
                        pos: other_pos,
                        value: other_value,
                    },
                ) => self_pos
                    .cmp(&other_pos)
                    .then_with(|| self_value.cmp(&other_value)),
                (
                    PruneCandidates {
                        pos: self_pos,
                        previous_candidates: self_previous_candidates,
                        remaining_candidates: self_remaining_candidates,
                    },
                    PruneCandidates {
                        pos: other_pos,
                        previous_candidates: other_previous_candidates,
                        remaining_candidates: other_remaining_candidates,
                    },
                ) => self_pos
                    .cmp(&other_pos)
                    .then_with(|| self_previous_candidates.cmp(&other_previous_candidates))
                    .then_with(|| self_remaining_candidates.cmp(&other_remaining_candidates)),
                (_, _) => Ordering::Equal,
            })
    }
}

impl<Base: SudokuBase> PartialOrd for OldDeduction<Base> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Base: SudokuBase> Display for OldDeduction<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OldDeduction::Value { pos, value } => {
                write!(f, "Value {value} at {pos}")
            }
            OldDeduction::PruneCandidates {
                pos,
                previous_candidates,
                remaining_candidates,
            } => {
                write!(f, "Prune candidates from {previous_candidates} to {remaining_candidates} at {pos}")
            }
        }
    }
}

impl<Base: SudokuBase> OldDeduction<Base> {
    pub fn apply(&self, grid: &mut Grid<Base>) {
        match *self {
            OldDeduction::Value { pos, value } => {
                grid.get_mut(pos).set_value(value);
                grid.update_candidates(pos, value);
            }
            OldDeduction::PruneCandidates {
                pos,
                previous_candidates,
                remaining_candidates,
            } => {
                let cell = grid.get_mut(pos);
                assert_eq!(cell.candidates(), Some(previous_candidates));
                cell.set_candidates(remaining_candidates);
            }
        }
    }

    pub fn position(&self) -> Position {
        match *self {
            OldDeduction::Value { pos, .. } | OldDeduction::PruneCandidates { pos, .. } => pos,
        }
    }

    fn discriminant_order(&self) -> u8 {
        match self {
            OldDeduction::Value { .. } => 0,
            OldDeduction::PruneCandidates { .. } => 1,
        }
    }

    // TODO: test
    pub fn postprocess(mut deductions: Vec<Self>) -> Vec<Self> {
        use itertools::Itertools;

        // Sort for dedup and ensure position ordering for group_by
        deductions.sort();
        // Remove duplicate deductions
        deductions.dedup();

        let deduction_position_groups =
            deductions.iter().group_by(|deduction| deduction.position());

        deduction_position_groups
            .into_iter()
            .filter_map(|(_pos, deduction_position_group)| {
                deduction_position_group
                    .into_iter()
                    .copied()
                    .reduce(|accum, deduction| {
                        accum
                            .merge(&deduction)
                            // TODO: propagate Result
                            .expect("Error while merging deductions")
                    })
            })
            .collect()
    }

    // TODO: move to constructors
    // TODO: validate against grid
    fn validate(&self) -> Result<()> {
        match self {
            OldDeduction::Value { .. } => {}
            OldDeduction::PruneCandidates {
                previous_candidates,
                remaining_candidates,
                ..
            } => {
                ensure!(
                    !previous_candidates.is_empty(),
                    "Unexpected PruneCandidates deduction for previously empty candidates: {self}"
                );
                ensure!(
                    !remaining_candidates.is_empty(),
                    "Unexpected PruneCandidates deduction removing all candidates: {self}"
                );
                ensure!(
                    previous_candidates != remaining_candidates,
                    "Unexpected no-op PruneCandidates deduction: {self}"
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
        use OldDeduction::*;

        self.validate()?;
        other.validate()?;

        if self == other {
            return Ok(*self);
        }

        let pos = self.position();

        ensure!(
            pos == other.position(),
            "Unexpected merge of two StrategyDeductions with different positions: {self}, {other}"
        );

        Ok(match (*self, *other) {
            (Value { .. }, Value { .. }) => bail!("Conflicting value deduction: {self}, {other}"),
            (
                PruneCandidates {
                    previous_candidates: self_previous_candidates,
                    ..
                },
                PruneCandidates {
                    previous_candidates: other_previous_candidates,
                    ..
                },
            ) if self_previous_candidates != other_previous_candidates => bail!("Conflicting previous_candidates: {self_previous_candidates}, {other_previous_candidates}"),
            // Merge PruneCandidates by intersecting their candidates.
            (
                PruneCandidates {
                    remaining_candidates: self_remaining_candidates,
                    previous_candidates,
                    ..
                },
                PruneCandidates {
                    remaining_candidates: other_remaining_candidates,
                    ..
                },
            ) => PruneCandidates {
                pos,
                previous_candidates,
                remaining_candidates: {
                    let intersected_candidates = self_remaining_candidates.intersection(&other_remaining_candidates);
                    ensure!(!intersected_candidates.is_empty(), "Conflicting remaining_candidates: {self_remaining_candidates}, {other_remaining_candidates}");
                    intersected_candidates
                },
            },
            // More specific Value overwrites PruneCandidates
            // TODO: ensure PruneCandidates is compatible with Value
            //  remaining_candidates must contain value
            (Value { pos,value }, _) | (_, Value { pos,value }) => Value {pos,value},
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;
    use crate::cell::Cell;
    use crate::samples;

    // TODO: port tests from OldDeduction to Deduction

    #[test]
    fn test_apply() {
        let mut grid = samples::base_2_candidates_coordinates();

        let pos = Position { row: 0, column: 1 };
        let value = 1.try_into().unwrap();
        OldDeduction::Value { pos, value }.apply(&mut grid);
        assert_eq!(*grid.get(pos), Cell::with_value(value, false));

        let pos = Position { row: 3, column: 3 };
        let candidates = vec![2, 4].try_into().unwrap();
        OldDeduction::PruneCandidates {
            pos,
            previous_candidates: Candidates::all(),
            remaining_candidates: candidates,
        }
        .apply(&mut grid);
        assert_eq!(
            *grid.get(pos),
            Cell::with_candidates(vec![2, 4].try_into().unwrap())
        );
    }

    #[test]
    fn test_postprocess() {
        use OldDeduction::*;

        let pos = Position { row: 1, column: 1 };
        let previous_candidates: Candidates<U2> = Candidates::all();
        let remaining_candidates: Candidates<U2> = Candidates::single(1.try_into().unwrap());

        let cases: Vec<(OldDeduction<U2>, OldDeduction<U2>, OldDeduction<U2>)> = vec![
            // Equal
            (
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
            ),
            // Left Value overwrites right PruneCandidates
            (
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
                PruneCandidates {
                    pos,
                    previous_candidates,
                    remaining_candidates,
                },
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
            ),
            // Right Value overwrites left PruneCandidates
            (
                PruneCandidates {
                    pos,
                    previous_candidates,
                    remaining_candidates,
                },
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
            ),
            // Intersect PruneCandidates
            (
                PruneCandidates {
                    pos,
                    previous_candidates,
                    remaining_candidates: vec![1, 2, 4].try_into().unwrap(),
                },
                PruneCandidates {
                    pos,
                    previous_candidates,
                    remaining_candidates: vec![1, 3, 4].try_into().unwrap(),
                },
                PruneCandidates {
                    pos,
                    previous_candidates,
                    remaining_candidates: vec![1, 4].try_into().unwrap(),
                },
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
    fn test_postprocess_err() {
        use OldDeduction::*;

        let pos = Position { row: 1, column: 1 };

        let err_cases: Vec<(OldDeduction<U2>, OldDeduction<U2>)> = vec![
            // Different pos
            (
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
                Value {
                    pos: Position { row: 2, column: 2 },
                    value: 1.try_into().unwrap(),
                },
            ),
            // Different value
            (
                Value {
                    pos,
                    value: 1.try_into().unwrap(),
                },
                Value {
                    pos,
                    value: 2.try_into().unwrap(),
                },
            ),
            // No intersection
            (
                PruneCandidates {
                    pos,
                    previous_candidates: Candidates::all(),
                    remaining_candidates: Candidates::single(1.try_into().unwrap()),
                },
                PruneCandidates {
                    pos,
                    previous_candidates: Candidates::all(),
                    remaining_candidates: Candidates::single(2.try_into().unwrap()),
                },
            ),
        ];

        for (left_strategy, right_strategy) in err_cases {
            assert!(left_strategy.merge(&right_strategy).is_err());
        }
    }
}
