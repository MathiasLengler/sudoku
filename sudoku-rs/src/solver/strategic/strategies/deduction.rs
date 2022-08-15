use super::*;
use crate::error::Result;
use anyhow::{bail, ensure};
use itertools::Itertools;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

// TODO: introduce wrapper struct
//  top-level fields:
//  - pos
//  - previous_candidates
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StrategyDeduction<Base: SudokuBase> {
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

impl<Base: SudokuBase> Ord for StrategyDeduction<Base> {
    fn cmp(&self, other: &Self) -> Ordering {
        use std::mem::discriminant;

        use StrategyDeduction::*;

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

impl<Base: SudokuBase> PartialOrd for StrategyDeduction<Base> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Base: SudokuBase> Display for StrategyDeduction<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyDeduction::Value { pos, value } => {
                write!(f, "Value {value} at {pos}")
            }
            StrategyDeduction::PruneCandidates {
                pos,
                previous_candidates,
                remaining_candidates,
            } => {
                write!(f, "Prune candidates from {previous_candidates} to {remaining_candidates} at {pos}")
            }
        }
    }
}

impl<Base: SudokuBase> StrategyDeduction<Base> {
    pub fn apply(&self, grid: &mut Grid<Base>) {
        match *self {
            StrategyDeduction::Value { pos, value } => {
                grid.get_mut(pos).set_value(value);
                grid.update_candidates(pos, value);
            }
            StrategyDeduction::PruneCandidates {
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
            StrategyDeduction::Value { pos, .. }
            | StrategyDeduction::PruneCandidates { pos, .. } => pos,
        }
    }

    fn discriminant_order(&self) -> u8 {
        match self {
            StrategyDeduction::Value { .. } => 0,
            StrategyDeduction::PruneCandidates { .. } => 1,
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
            StrategyDeduction::Value { .. } => {}
            StrategyDeduction::PruneCandidates {
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
        use StrategyDeduction::*;

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

    #[test]
    fn test_apply() {
        let mut grid = samples::base_2_candidates_coordinates();

        let pos = Position { row: 0, column: 1 };
        let value = 1.try_into().unwrap();
        StrategyDeduction::Value { pos, value }.apply(&mut grid);
        assert_eq!(*grid.get(pos), Cell::with_value(value, false));

        let pos = Position { row: 3, column: 3 };
        let candidates = vec![2, 4].try_into().unwrap();
        StrategyDeduction::PruneCandidates {
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
        use StrategyDeduction::*;

        let pos = Position { row: 1, column: 1 };
        let previous_candidates: Candidates<U2> = Candidates::all();
        let remaining_candidates: Candidates<U2> = Candidates::single(1.try_into().unwrap());

        let cases: Vec<(
            StrategyDeduction<U2>,
            StrategyDeduction<U2>,
            StrategyDeduction<U2>,
        )> = vec![
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
        use StrategyDeduction::*;

        let pos = Position { row: 1, column: 1 };

        let err_cases: Vec<(StrategyDeduction<U2>, StrategyDeduction<U2>)> = vec![
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
