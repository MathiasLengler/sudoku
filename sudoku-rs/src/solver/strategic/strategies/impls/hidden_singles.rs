use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::grid::group::Group;
use crate::position::Position;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions};
use crate::solver::strategic::strategies::{Strategy, StrategyScore};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CandidateStats<Base: SudokuBase> {
    count: u8,
    last_pos: Option<Position<Base>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HiddenSingles;

impl Strategy for HiddenSingles {
    fn name(self) -> &'static str {
        "HiddenSingles"
    }
    fn score(self) -> StrategyScore {
        10
    }
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Ok(Grid::<Base>::all_group_positions()
            .flat_map(|group_positions| {
                let mut candidate_histogram = Group::<Base, CandidateStats<Base>>::default();

                for group_position in group_positions {
                    if let Some(candidates) = grid.get(group_position).candidates() {
                        for candidate in candidates {
                            let bucket = candidate_histogram.get_mut(candidate.into());
                            bucket.count += 1;
                            bucket.last_pos = Some(group_position);
                        }
                    }
                }

                candidate_histogram
                    .into_iter_enumerate()
                    .filter(|&(_coordinate, stats)| stats.count == 1)
                    .map(|(coordinate, stats)| {
                        // This candidate is unique in this group.
                        let pos = stats.last_pos.unwrap();

                        Deduction::with_action(pos, Action::SetValue(coordinate.into()))
                    })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Value;
    use crate::samples;
    use crate::solver::strategic::strategies::test_util::assert_deductions_with_grid;
    use crate::{
        base::consts::*, solver::strategic::strategies::test_util::strategy_snapshot_tests,
    };

    #[test]
    fn test_hidden_singles_base2() {
        let mut grid = samples::base_2().into_iter().nth(1).unwrap();

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let deductions = HiddenSingles.execute(&grid).unwrap();

        let expected_deductions: Deductions<_> = vec![
            //
            ((0, 1), 2),
            ((1, 2), 2),
            ((2, 3), 4),
            ((3, 0), 4),
        ]
        .into_iter()
        .map(|(pos, value)| {
            Deduction::with_action(
                pos.try_into().unwrap(),
                Action::SetValue(Value::try_from(value).unwrap()),
            )
        })
        .collect();

        assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
    }

    #[test]
    fn test_hidden_singles_base3() {
        let mut grid: Grid<Base3> =
            "000000300000071500002400018000009040094618230610700000430897600008140000009000000"
                .parse()
                .unwrap();
        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let deductions = HiddenSingles.execute(&grid).unwrap();

        let expected_deductions: Deductions<_> = vec![
            //
            ((0, 4), 8),
            ((3, 8), 6),
            ((5, 5), 4),
            ((8, 6), 4),
        ]
        .into_iter()
        .map(|(pos, value)| {
            Deduction::with_action(
                pos.try_into().unwrap(),
                Action::SetValue(Value::try_from(value).unwrap()),
            )
        })
        .collect();

        assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
    }

    strategy_snapshot_tests!(HiddenSingles);
}
