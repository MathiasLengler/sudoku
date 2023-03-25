pub use action::Action;
pub use deduction::Deduction;
pub use deductions::Deductions;
pub use position_map::{Merge, PositionMap};
pub use reason::Reason;
pub use transport::{TransportAction, TransportDeduction, TransportReason};

mod action;
#[allow(clippy::module_inception)]
mod deduction;
mod deductions;
mod position_map;
mod reason;
mod transport;

#[cfg(test)]
mod tests {
    // TODO: port test to new Deductions

    // use crate::base::consts::*;
    // #[test]
    // fn test_deductions_order_independence() {
    //     use itertools::Itertools;
    //
    //     let pos = DynamicPosition { row: 0, column: 0 };
    //     let previous_candidates: Candidates<Base2> = Candidates::all();
    //     let remaining_candidates: Candidates<Base2> = vec![1, 2].try_into().unwrap();
    //
    //     let value_deduction_1 =
    //         OldDeduction::with_value(pos, previous_candidates, 1.try_into().unwrap()).unwrap();
    //     let value_deduction_2 = OldDeduction::with_value(
    //         DynamicPosition { row: 1, column: 1 },
    //         previous_candidates,
    //         2.try_into().unwrap(),
    //     )
    //     .unwrap();
    //     let remaining_candidates_deduction =
    //         OldDeduction::with_remaining_candidates(pos, previous_candidates, remaining_candidates)
    //             .unwrap();
    //
    //     let all_deductions: Vec<OldDeduction<Base2>> = vec![
    //         value_deduction_1,
    //         value_deduction_2,
    //         remaining_candidates_deduction,
    //     ];
    //     let deductions: OldDeductions<Base2> =
    //         IntoDeductions(all_deductions.clone()).try_into().unwrap();
    //     for deduction_permutation in all_deductions.into_iter().permutations(3) {
    //         assert_eq!(
    //             OldDeductions::<Base2>::try_from(IntoDeductions(deduction_permutation)).unwrap(),
    //             deductions
    //         );
    //     }
    // }
    //
    // #[test]
    // fn test_deduction_apply() {
    //     let mut grid = samples::base_2_candidates_coordinates();
    //
    //     let pos = DynamicPosition { row: 0, column: 1 };
    //     let value = 1.try_into().unwrap();
    //     OldDeduction::with_value(pos, Candidates::single(1.try_into().unwrap()), value)
    //         .unwrap()
    //         .apply(&mut grid);
    //     assert_eq!(*grid.get(pos), Cell::with_value(value, false));
    //
    //     let pos = DynamicPosition { row: 3, column: 3 };
    //     let candidates = vec![2, 4].try_into().unwrap();
    //     OldDeduction::with_remaining_candidates(pos, Candidates::all(), candidates)
    //         .unwrap()
    //         .apply(&mut grid);
    //     assert_eq!(*grid.get(pos), Cell::with_candidates(candidates));
    // }
    //
    // #[test]
    // fn test_deduction_merge() {
    //     let pos = DynamicPosition { row: 1, column: 1 };
    //     let previous_candidates: Candidates<Base2> = Candidates::all();
    //     let remaining_candidates: Candidates<Base2> = Candidates::single(1.try_into().unwrap());
    //     let value: Value<Base2> = 1.try_into().unwrap();
    //
    //     let cases: Vec<(
    //         OldDeduction<Base2>,
    //         OldDeduction<Base2>,
    //         OldDeduction<Base2>,
    //     )> = vec![
    //         // Equal
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //         ),
    //         // Left Value overwrites right PruneCandidates
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //         ),
    //         // Right Value overwrites left PruneCandidates
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //         ),
    //         // Intersect PruneCandidates
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 vec![1, 2, 4].try_into().unwrap(),
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 vec![1, 3, 4].try_into().unwrap(),
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 vec![1, 4].try_into().unwrap(),
    //             )
    //             .unwrap(),
    //         ),
    //     ];
    //
    //     for (left_strategy, right_strategy, expected_strategy) in cases {
    //         assert_eq!(
    //             left_strategy.merge(&right_strategy).unwrap(),
    //             expected_strategy
    //         );
    //     }
    // }
    // #[test]
    // fn test_deduction_merge_err() {
    //     let pos = DynamicPosition { row: 1, column: 1 };
    //     let different_pos = DynamicPosition { row: 2, column: 2 };
    //     let previous_candidates: Candidates<Base2> = Candidates::all();
    //     let different_previous_candidates: Candidates<Base2> = vec![1, 2].try_into().unwrap();
    //     let remaining_candidates: Candidates<Base2> = Candidates::single(1.try_into().unwrap());
    //     let different_remaining_candidates: Candidates<Base2> =
    //         Candidates::single(2.try_into().unwrap());
    //     let value: Value<Base2> = 1.try_into().unwrap();
    //     let different_value: Value<Base2> = 2.try_into().unwrap();
    //
    //     let err_cases: Vec<(OldDeduction<Base2>, OldDeduction<Base2>)> = vec![
    //         // Different pos
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(different_pos, previous_candidates, value).unwrap(),
    //         ),
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 different_pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //         ),
    //         // Different previous_candidates
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, different_previous_candidates, value).unwrap(),
    //         ),
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 different_previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //         ),
    //         // Different value
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, different_value).unwrap(),
    //         ),
    //         // No intersection
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 different_remaining_candidates,
    //             )
    //             .unwrap(),
    //         ),
    //     ];
    //
    //     for (left_strategy, right_strategy) in err_cases {
    //         assert!(left_strategy.merge(&right_strategy).is_err());
    //     }
    // }
}
