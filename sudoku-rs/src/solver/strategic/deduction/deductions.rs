use std::collections::{btree_set, BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::PositionMap;
use crate::solver::strategic::deduction::transport::TransportDeductions;
use crate::solver::strategic::deduction::{Action, Deduction, Reason};

/// A list of deductions made by a strategy.
/// Some strategies can be applied multiple times on a single grid, e.g.:
/// - multiple hidden singles
/// - multiple pairs
/// - multiple distinct X-Wings
///
/// Strategies are encouraged to report logically separate deductions as multiple instances of `Deduction`,
/// in order to enable:
/// - application of single `Deductions`
/// - clear distinction of the reasoning for each `Deduction`
/// - enabling the hint UI to only reveal a single `Deduction`
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Deductions<Base: SudokuBase> {
    deductions: BTreeSet<Deduction<Base>>,
}

impl<Base: SudokuBase> TryFrom<TransportDeductions> for Deductions<Base> {
    type Error = Error;

    fn try_from(transport_deductions: TransportDeductions) -> Result<Self> {
        let TransportDeductions { deductions } = transport_deductions;
        Ok(Self {
            deductions: deductions
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
        })
    }
}

impl<Base: SudokuBase> Display for Deductions<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use itertools::Itertools;

        write!(
            f,
            "{}",
            self.deductions
                .iter()
                .map(|deduction| deduction.to_string())
                .join("\n")
        )
    }
}

impl<Base: SudokuBase> Deductions<Base> {
    pub fn iter(&self) -> btree_set::Iter<'_, Deduction<Base>> {
        self.deductions.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.deductions.is_empty()
    }

    pub fn count(&self) -> usize {
        self.deductions.len()
    }

    fn as_merged_deduction(&self) -> Result<Deduction<Base>> {
        let mut merged_deduction = Deduction::default();

        for deduction in &self.deductions {
            for (pos, action) in deduction.actions.iter() {
                merged_deduction.actions.insert(pos, *action)?;
            }
            for (pos, reasons) in deduction.reasons.iter() {
                merged_deduction.reasons.insert(pos, *reasons)?;
            }
        }
        Ok(merged_deduction)
    }

    /// If two deductions contain the same reasons, merge them into a single deduction by merging their actions.
    pub fn merge_deductions_by_reasons(self) -> Result<Self> {
        let mut reasons_to_actions: BTreeMap<
            PositionMap<Base, Reason<Base>>,
            PositionMap<Base, Action<Base>>,
        > = BTreeMap::new();

        for Deduction { reasons, actions } in self {
            if let Some(existing_actions) = reasons_to_actions.get_mut(&reasons) {
                existing_actions.merge(actions)?;
            } else {
                reasons_to_actions.insert(reasons, actions);
            }
        }

        Ok(reasons_to_actions
            .into_iter()
            .map(|(reasons, actions)| Deduction { actions, reasons })
            .collect())
    }

    /// If two deductions contain the same actions, merge them into a single deduction by merging their reasons.
    pub fn merge_deductions_by_actions(self) -> Result<Self> {
        let mut actions_to_reasons: BTreeMap<
            PositionMap<Base, Action<Base>>,
            PositionMap<Base, Reason<Base>>,
        > = BTreeMap::new();

        for Deduction { reasons, actions } in self {
            if let Some(existing_reasons) = actions_to_reasons.get_mut(&actions) {
                existing_reasons.merge(reasons)?;
            } else {
                actions_to_reasons.insert(actions, reasons);
            }
        }

        Ok(actions_to_reasons
            .into_iter()
            .map(|(actions, reasons)| Deduction { actions, reasons })
            .collect())
    }

    fn validate(&self, grid: &Grid<Base>) -> Result<()> {
        for deduction in &self.deductions {
            deduction.validate(grid)?;
        }
        Ok(())
    }

    pub fn apply(&self, grid: &mut Grid<Base>) -> Result<()> {
        self.validate(grid)?;

        let merged_deduction = self.as_merged_deduction()?;
        merged_deduction.apply(grid)?;

        Ok(())
    }
}

impl<'a, Base: SudokuBase> IntoIterator for &'a Deductions<Base> {
    type Item = &'a Deduction<Base>;
    type IntoIter = btree_set::Iter<'a, Deduction<Base>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Base: SudokuBase> IntoIterator for Deductions<Base> {
    type Item = Deduction<Base>;
    type IntoIter = btree_set::IntoIter<Deduction<Base>>;

    fn into_iter(self) -> Self::IntoIter {
        self.deductions.into_iter()
    }
}

impl<Base: SudokuBase> FromIterator<Deduction<Base>> for Deductions<Base> {
    fn from_iter<T: IntoIterator<Item = Deduction<Base>>>(iter: T) -> Self {
        Self {
            deductions: iter.into_iter().collect(),
        }
    }
}
