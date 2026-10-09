//! Locked sets via Régin's filtering for one `alldifferent` constraint.
//!
//! Régin 1994 (Thm 1): a candidate survives iff its cell–value edge lies in some
//! perfect matching of the group's value graph. Berge (Régin Prop. 1): for a
//! fixed matching `M`, such an edge is either in `M` or on an even alternating
//! cycle, i.e. a directed cycle of the graph oriented cell → `M(cell)` and
//! value → cells holding it unmatched. Edges on even alternating paths from a
//! free vertex (Régin Alg. 2 step 2) cannot occur here: a group has as many
//! cells as values, so a matching covering the cells covers the values too.
//!
//! Cells contract away since each has the single successor `M(cell)`. On values
//! alone, `succ[v] = { M(c) | c ∋ v, M(c) ≠ v }`, and a candidate `v` of cell
//! `c` survives iff `reach[M(c)]` contains `v`, because `v → c → M(c)` closes
//! the cycle. `reach` is the reflexive-transitive closure over `N` bitsets.
//!
//! Each strongly connected component pairs a value set with the cells matched
//! to it. A component without incoming cross edges is a naked set, one without
//! outgoing cross edges a hidden set. Interior components only become visible
//! to a human after the outer sets are applied.
//!
//! References: van Hoeve 2001 §6 (survey, arXiv cs/0105015); Régin 1994 (AAAI-94).

use anyhow::bail;

use crate::base::SudokuBase;
use crate::cell::{Candidates, Value, map::ValueMap};
use crate::error::Error;
use crate::grid::group::{CandidatesGroup, Group};
use crate::position::Coordinate;

/// What a human can see in the current candidates.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LockedSetKind {
    Naked,
    Hidden,
}

/// Every strongly connected component. `Interior` eliminations are only
/// justified transitively; they turn naked or hidden once the outer sets are applied.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ComponentKind {
    Naked,
    Hidden,
    Interior,
}

impl From<LockedSetKind> for ComponentKind {
    fn from(kind: LockedSetKind) -> Self {
        match kind {
            LockedSetKind::Naked => Self::Naked,
            LockedSetKind::Hidden => Self::Hidden,
        }
    }
}

impl TryFrom<ComponentKind> for LockedSetKind {
    type Error = Error;

    fn try_from(kind: ComponentKind) -> Result<Self, Self::Error> {
        Ok(match kind {
            ComponentKind::Naked => Self::Naked,
            ComponentKind::Hidden => Self::Hidden,
            ComponentKind::Interior => bail!("interior component is not a locked set"),
        })
    }
}

/// A set of group cells locked to a set of values of the same size.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LockedSet<Base: SudokuBase, K> {
    pub kind: K,
    /// Group coordinates.
    pub cells: Candidates<Base>,
    pub values: Candidates<Base>,
}

impl<Base: SudokuBase, K> LockedSet<Base, K> {
    /// Candidates to delete per cell: outside cells lose `values`, set cells keep only `values`.
    pub fn eliminations<'a>(
        &'a self,
        group: &'a CandidatesGroup<Base>,
    ) -> impl Iterator<Item = (Coordinate<Base>, Candidates<Base>)> + 'a {
        group
            .iter_enumerate()
            .filter_map(move |(coordinate, candidates)| {
                let deleted = if self.cells.has(coordinate) {
                    candidates.without(self.values)
                } else {
                    candidates.intersection(self.values)
                };
                deleted.is_non_empty().then_some((coordinate, deleted))
            })
    }

    /// Kept candidates per set cell.
    pub fn reasons<'a>(
        &'a self,
        group: &'a CandidatesGroup<Base>,
    ) -> impl Iterator<Item = (Coordinate<Base>, Candidates<Base>)> + 'a {
        group
            .iter_enumerate()
            .filter(move |(coordinate, _candidates)| self.cells.has(*coordinate))
            .map(move |(coordinate, candidates)| (coordinate, candidates.intersection(self.values)))
    }

    #[must_use]
    pub fn apply(&self, group: &CandidatesGroup<Base>) -> CandidatesGroup<Base> {
        let mut reduced = group.clone();
        for (coordinate, deleted) in self.eliminations(group) {
            let candidates = reduced.get_mut(coordinate);
            *candidates = candidates.without(deleted);
        }
        reduced
    }
}

/// Smallest naked or hidden set that eliminates something; naked before hidden,
/// then lowest first cell. `None` if nothing can be eliminated or the group has
/// no perfect matching (contradictory, left to other strategies).
pub fn find_locked_set<Base: SudokuBase>(
    group: &CandidatesGroup<Base>,
) -> Option<LockedSet<Base, LockedSetKind>> {
    let analysis = Analysis::new(group)?;
    analysis
        .components()
        .filter_map(|component| {
            LockedSetKind::try_from(component.kind)
                .ok()
                .map(|kind| LockedSet {
                    kind,
                    cells: component.cells,
                    values: component.values,
                })
        })
        .min_by_key(|set| {
            (
                set.values.count(),
                set.kind,
                set.cells.first::<Coordinate<Base>>(),
            )
        })
}

/// Every component that eliminates something. Applying all yields [`reduce`].
pub fn find_locked_sets<Base: SudokuBase>(
    group: &CandidatesGroup<Base>,
) -> Vec<LockedSet<Base, ComponentKind>> {
    Analysis::new(group).map_or_else(Vec::new, |analysis| analysis.components().collect())
}

/// The hyper-arc consistent group: every candidate in some perfect matching.
/// Equals [`super::LockedSets::find_locked_set_v1`] whenever a perfect matching exists;
/// without one the group is returned unchanged.
#[must_use]
pub fn reduce<Base: SudokuBase>(group: &CandidatesGroup<Base>) -> CandidatesGroup<Base> {
    Analysis::new(group).map_or_else(|| group.clone(), |analysis| analysis.reduce(group))
}

struct Analysis<Base: SudokuBase> {
    /// Cell → matched value.
    matching: Group<Base, Value<Base>>,
    /// Values reachable from a value, reflexive.
    reach: ValueMap<Base, Candidates<Base>>,
    /// Values a value is reachable from, reflexive.
    reached_by: ValueMap<Base, Candidates<Base>>,
}

impl<Base: SudokuBase> Analysis<Base> {
    fn new(group: &CandidatesGroup<Base>) -> Option<Self> {
        let matching = perfect_matching(group)?;

        let mut reach: ValueMap<Base, Candidates<Base>> = ValueMap::default();
        for value in Value::<Base>::all() {
            reach.get_mut(value).insert(value);
        }
        for (cell, candidates) in group.iter_enumerate() {
            let matched = matching.get(cell);
            for value in candidates.without(Candidates::with_single(matched)) {
                reach.get_mut(value).insert(matched);
            }
        }

        // Warshall; row k is stable during its own round.
        for k in Value::<Base>::all() {
            let reach_k = reach.get(k);
            for v in Value::<Base>::all() {
                let reach_v = reach.get_mut(v);
                if reach_v.has(k) {
                    *reach_v = reach_v.union(reach_k);
                }
            }
        }

        let mut reached_by: ValueMap<Base, Candidates<Base>> = ValueMap::default();
        for u in Value::<Base>::all() {
            for v in reach.get(u) {
                reached_by.get_mut(v).insert(u);
            }
        }

        Some(Self {
            matching,
            reach,
            reached_by,
        })
    }

    fn components(&self) -> impl Iterator<Item = LockedSet<Base, ComponentKind>> + '_ {
        let mut seen = Candidates::<Base>::new();
        Value::<Base>::all().filter_map(move |value| {
            if seen.has(value) {
                return None;
            }
            let reach = self.reach.get(value);
            let reached_by = self.reached_by.get(value);
            let values = reach.intersection(reached_by);
            seen = seen.union(values);

            let kind = match (reached_by == values, reach == values) {
                (true, true) => return None,
                (true, false) => ComponentKind::Naked,
                (false, true) => ComponentKind::Hidden,
                (false, false) => ComponentKind::Interior,
            };
            let cells = self
                .matching
                .iter_enumerate()
                .filter(|(_cell, matched)| values.has(*matched))
                .map(|(cell, _matched)| cell)
                .collect();

            Some(LockedSet {
                kind,
                cells,
                values,
            })
        })
    }

    fn reduce(&self, group: &CandidatesGroup<Base>) -> CandidatesGroup<Base> {
        let mut reduced = group.clone();
        for (cell, candidates) in reduced.iter_mut_enumerate() {
            *candidates = candidates.intersection(self.reach.get(self.matching.get(cell)));
        }
        reduced
    }
}

/// Kuhn's augmenting paths after a greedy start; N ≤ 25 makes Hopcroft–Karp pointless.
fn perfect_matching<Base: SudokuBase>(
    group: &CandidatesGroup<Base>,
) -> Option<Group<Base, Value<Base>>> {
    let mut value_of_cell: Group<Base, Option<Value<Base>>> = Group::with_all(None);
    let mut cell_of_value: ValueMap<Base, Option<Coordinate<Base>>> = ValueMap::with_all(None);

    for (cell, candidates) in group.iter_enumerate() {
        if let Some(value) = candidates
            .iter()
            .find(|value| cell_of_value.get(*value).is_none())
        {
            *value_of_cell.get_mut(cell) = Some(value);
            *cell_of_value.get_mut(value) = Some(cell);
        }
    }

    for cell in Coordinate::<Base>::all() {
        if value_of_cell.get(cell).is_some() {
            continue;
        }
        let mut visited = Candidates::new();
        if !augment(
            group,
            cell,
            &mut visited,
            &mut value_of_cell,
            &mut cell_of_value,
        ) {
            return None;
        }
    }

    let mut matching = Group::with_all(Value::<Base>::max());
    for (cell, value) in value_of_cell.iter_enumerate() {
        *matching.get_mut(cell) = value?;
    }
    Some(matching)
}

fn augment<Base: SudokuBase>(
    group: &CandidatesGroup<Base>,
    cell: Coordinate<Base>,
    visited: &mut Candidates<Base>,
    value_of_cell: &mut Group<Base, Option<Value<Base>>>,
    cell_of_value: &mut ValueMap<Base, Option<Coordinate<Base>>>,
) -> bool {
    for value in group.get(cell) {
        if visited.has(value) {
            continue;
        }
        visited.insert(value);

        let free = match cell_of_value.get(value) {
            None => true,
            Some(other) => augment(group, other, visited, value_of_cell, cell_of_value),
        };
        if free {
            *value_of_cell.get_mut(cell) = Some(value);
            *cell_of_value.get_mut(value) = Some(cell);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;
    use crate::rng::new_crate_rng_with_seed;
    use crate::solver::strategic::strategies::LockedSets;
    use crate::solver::strategic::strategies::locked_sets::bench_cases::bench_cases;
    use crate::solver::strategic::strategies::locked_sets::v2::test_utils::{
        TestCase, locked_set_test_cases_base_2, locked_set_test_cases_base_3,
    };
    use crate::test_util::init_test_logger;
    use itertools::Itertools;
    use log::info;
    use rand::prelude::*;
    use std::iter::repeat_n;

    fn v1<Base: SudokuBase>(group: &CandidatesGroup<Base>) -> CandidatesGroup<Base> {
        CandidatesGroup::try_from(LockedSets::find_locked_set_v1(group.as_slice())).unwrap()
    }

    fn fixpoint<Base: SudokuBase>(group: &CandidatesGroup<Base>) -> CandidatesGroup<Base> {
        let mut group = group.clone();
        while let Some(set) = find_locked_set(&group) {
            let reduced = set.apply(&group);
            assert_ne!(
                reduced.as_slice(),
                group.as_slice(),
                "set without eliminations {set:?} in\n{group}"
            );
            group = reduced;
        }
        group
    }

    fn apply_all<Base: SudokuBase>(group: &CandidatesGroup<Base>) -> CandidatesGroup<Base> {
        find_locked_sets(group)
            .iter()
            .fold(group.clone(), |group, set| set.apply(&group))
    }

    /// All three v3 outputs agree with v1 on a feasible group.
    fn assert_matches_v1<Base: SudokuBase>(group: &CandidatesGroup<Base>) {
        let expected = v1(group);
        let reduced = reduce(group);
        assert_eq!(
            reduced.as_slice(),
            expected.as_slice(),
            "\nInput:\n{group}V1:\n{expected}!= reduce:\n{reduced}"
        );
        let all = apply_all(group);
        assert_eq!(
            all.as_slice(),
            expected.as_slice(),
            "\nInput:\n{group}V1:\n{expected}!= apply_all:\n{all}"
        );
        let stepwise = fixpoint(group);
        assert_eq!(
            stepwise.as_slice(),
            expected.as_slice(),
            "\nInput:\n{group}V1:\n{expected}!= fixpoint:\n{stepwise}"
        );
    }

    fn assert_test_case<Base: SudokuBase>((name, input, expected): &TestCase<Base>)
    where
        Base::Group<Candidates<Base>>: PartialEq,
    {
        info!("Test case: {name}");
        let actual = find_locked_set(input).map_or_else(|| input.clone(), |set| set.apply(input));
        if &actual != expected {
            // v2 picks the lexicographically first set among equals; only require v3's step to be sound.
            assert_ne!(&actual, input, "{name}: v2 found a set, v3 none:\n{input}");
            assert_eq!(
                fixpoint(&actual).as_slice(),
                v1(input).as_slice(),
                "{name}: unsound step\n{actual}"
            );
            info!("{name}: different tie-break than v2:\n{actual}");
        }
    }

    #[test]
    fn test_find_locked_set_base_2() {
        init_test_logger();
        for test_case in locked_set_test_cases_base_2() {
            assert_test_case(&test_case);
        }
    }

    #[test]
    fn test_find_locked_set_base_3() {
        init_test_logger();
        for test_case in locked_set_test_cases_base_3() {
            assert_test_case(&test_case);
        }
    }

    #[test]
    fn test_vs_v1_base_2_exhaustive() {
        type Base = Base2;

        let mut feasible = 0;
        for input in repeat_n(Candidates::<Base>::iter_all_lexicographical(), 4)
            .multi_cartesian_product()
            .map(|group| CandidatesGroup::<Base>::try_from(group).unwrap())
        {
            if input.iter().any(|candidates| candidates.is_empty()) {
                continue;
            }
            if v1(&input).iter().any(|candidates| candidates.is_empty()) {
                assert!(find_locked_set(&input).is_none(), "infeasible:\n{input}");
                assert!(find_locked_sets(&input).is_empty(), "infeasible:\n{input}");
                assert_eq!(reduce(&input).as_slice(), input.as_slice());
                continue;
            }
            assert_matches_v1(&input);
            feasible += 1;
        }
        assert!(feasible > 0);
    }

    #[test]
    fn test_vs_v1_base_3_random() {
        type Base = Base3;

        let mut rng = new_crate_rng_with_seed(Some(3));
        let values = Value::<Base>::all().collect_vec();
        let mut feasible = 0;
        for _ in 0..3000 {
            let input = CandidatesGroup::<Base>::try_from(
                (0..Base::SIDE_LENGTH)
                    .map(|_| {
                        let count = rng.random_range(1..=5);
                        values
                            .sample(&mut rng, count)
                            .copied()
                            .collect::<Candidates<Base>>()
                    })
                    .collect_vec(),
            )
            .unwrap();
            if v1(&input).iter().any(|candidates| candidates.is_empty()) {
                assert!(find_locked_set(&input).is_none(), "infeasible:\n{input}");
                continue;
            }
            assert_matches_v1(&input);
            feasible += 1;
        }
        assert!(feasible > 100, "only {feasible} feasible groups");
    }

    fn assert_bench_cases<Base: SudokuBase>() {
        for case in bench_cases::<Base>() {
            assert_eq!(
                find_locked_set(&case.candidates_group).is_some(),
                case.has_locked_set,
                "{}:\n{}",
                case.name,
                case.candidates_group
            );
            if case.v1_feasible {
                assert_matches_v1(&case.candidates_group);
            } else {
                assert_eq!(
                    apply_all(&case.candidates_group).as_slice(),
                    reduce(&case.candidates_group).as_slice()
                );
                assert_eq!(
                    fixpoint(&case.candidates_group).as_slice(),
                    reduce(&case.candidates_group).as_slice()
                );
            }
        }
    }

    #[test]
    fn test_bench_cases_base_2() {
        assert_bench_cases::<Base2>();
    }

    #[test]
    fn test_bench_cases_base_3() {
        assert_bench_cases::<Base3>();
    }

    #[test]
    fn test_bench_cases_base_4() {
        assert_bench_cases::<Base4>();
    }

    #[test]
    fn test_bench_cases_base_5() {
        assert_bench_cases::<Base5>();
    }

    /// Chain of singles: the middle component is neither naked nor hidden yet.
    #[test]
    fn test_interior_component() {
        type Base = Base2;
        let group = CandidatesGroup::<Base>::try_from(
            [vec![1u8], vec![1, 2], vec![2, 3], vec![2, 3, 4]]
                .into_iter()
                .map(|values| Candidates::try_from(values).unwrap())
                .collect_vec(),
        )
        .unwrap();

        let kinds = find_locked_sets(&group)
            .into_iter()
            .map(|set| set.kind)
            .collect_vec();
        assert_eq!(
            kinds,
            [
                ComponentKind::Naked,
                ComponentKind::Interior,
                ComponentKind::Interior,
                ComponentKind::Hidden
            ]
        );

        let set = find_locked_set(&group).unwrap();
        assert_eq!(set.kind, LockedSetKind::Naked);
        assert_eq!(set.cells.to_vec_u8(), [1]);
        assert_eq!(set.values.to_vec_u8(), [1]);
        assert_eq!(
            set.eliminations(&group).collect_vec(),
            [(
                Coordinate::new(1).unwrap(),
                Candidates::try_from(vec![1u8]).unwrap()
            )]
        );
        assert_eq!(
            set.reasons(&group).collect_vec(),
            [(
                Coordinate::new(0).unwrap(),
                Candidates::try_from(vec![1u8]).unwrap()
            )]
        );
    }
}
