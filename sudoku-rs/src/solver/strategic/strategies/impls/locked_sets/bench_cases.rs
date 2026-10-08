//! Generated `find_locked_set` inputs with the same shapes at every base, for benchmarks.

use crate::base::SudokuBase;
use crate::grid::group::CandidatesGroup;
use crate::solver::strategic::strategies::locked_sets::v2::test_utils::candidates_group_from_test_data;

#[derive(Debug, Clone)]
pub struct BenchCase<Base: SudokuBase> {
    pub name: String,
    /// `find_locked_set` reduces the input iff set.
    pub has_locked_set: bool,
    /// Whether `find_locked_set_v1` finishes in bench time.
    /// It enumerates perfect matchings, so cells with most values in them cost factorially:
    /// fine up to 8! at Base3, hopeless from 12! at Base4. Observed once, then hard-coded.
    pub v1_feasible: bool,
    pub candidates_group: CandidatesGroup<Base>,
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Naked,
    Hidden,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Self::Naked => "Naked",
            Self::Hidden => "Hidden",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SetShape {
    /// Every set cell holds all set values.
    Complete,
    /// Set cells hold cyclic pairs of the set values.
    Distributed,
}

impl SetShape {
    fn label(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Distributed => "distributed",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Outside {
    /// Every outside cell holds all values it may hold.
    Filled,
    /// Bivalue cycle over the free values.
    Sparse,
    /// Short bivalue cycle, remaining outside cells solved.
    Singles,
}

impl Outside {
    fn label(self) -> &'static str {
        match self {
            Self::Filled => "filled",
            Self::Sparse => "sparse",
            Self::Singles => "singles",
        }
    }
}

const SET_SIZE_NAMES: [&str; 4] = ["single", "pair", "triple", "quad"];

pub fn bench_cases<Base: SudokuBase>() -> Vec<BenchCase<Base>> {
    let n = Base::MAX_VALUE;
    let mut cases = Vec::new();

    for set_size in 1..=4.min(n / 2) {
        for kind in [Kind::Naked, Kind::Hidden] {
            for shape in [SetShape::Complete, SetShape::Distributed] {
                // Cyclic pairs of fewer than 3 values are the complete set
                if matches!(shape, SetShape::Distributed) && set_size < 3 {
                    continue;
                }
                for outside in [Outside::Filled, Outside::Sparse, Outside::Singles] {
                    // Singles needs an outside cell to solve
                    if matches!(outside, Outside::Singles)
                        && singles_cycle_len(n, set_size) >= n - set_size
                    {
                        continue;
                    }
                    cases.push(BenchCase {
                        name: format!(
                            "{} {} - {} - outside {}",
                            kind.label(),
                            SET_SIZE_NAMES[usize::from(set_size) - 1],
                            shape.label(),
                            outside.label()
                        ),
                        has_locked_set: true,
                        v1_feasible: n <= 9 || !matches!(outside, Outside::Filled),
                        candidates_group: candidates_group_from_test_data(locked_set_group(
                            n, set_size, kind, shape, outside,
                        )),
                    });
                }
            }
        }
    }

    let mut no_set = |name: &str, width: u8| {
        cases.push(BenchCase {
            name: format!("No set - {name}"),
            has_locked_set: false,
            v1_feasible: n <= 9 || width == 2,
            candidates_group: candidates_group_from_test_data(cyclic_group(n, width)),
        });
    };
    no_set("bivalue cycle", 2);
    if n / 2 > 2 {
        no_set("half cycle", n / 2);
    }
    no_set("all candidates", n);

    cases
}

/// Set cells first, then outside cells. Values of the other side are injected
/// (all of them into filled cells, otherwise the `j`-th into cell `j`) so the set eliminates something.
fn locked_set_group(
    n: u8,
    set_size: u8,
    kind: Kind,
    shape: SetShape,
    outside: Outside,
) -> Vec<Vec<u8>> {
    let set_values: Vec<u8> = (1..=set_size).map(|j| 2 * j).collect();
    let free_values: Vec<u8> = (1..=n)
        .filter(|value| !set_values.contains(value))
        .collect();
    let singles_cycle_len = usize::from(singles_cycle_len(n, set_size));

    let set_cells = (0..usize::from(set_size)).map(|j| match shape {
        SetShape::Complete => set_values.clone(),
        SetShape::Distributed => cycle_pair(&set_values, j),
    });
    let free_cells = (0..free_values.len()).map(|i| match outside {
        Outside::Filled => free_values.clone(),
        Outside::Sparse => cycle_pair(&free_values, i),
        Outside::Singles if i < singles_cycle_len => {
            cycle_pair(&free_values[..singles_cycle_len], i)
        }
        Outside::Singles => vec![free_values[i]],
    });

    let filled = matches!(outside, Outside::Filled);
    let inject = |cells: &mut Vec<Vec<u8>>, values: &[u8]| {
        for (j, cell) in cells.iter_mut().enumerate() {
            if filled {
                cell.extend_from_slice(values);
            } else if let Some(&value) = values.get(j) {
                cell.push(value);
            }
        }
    };

    let mut set_cells: Vec<Vec<u8>> = set_cells.collect();
    let mut free_cells: Vec<Vec<u8>> = free_cells.collect();
    match kind {
        Kind::Naked => inject(&mut free_cells, &set_values),
        Kind::Hidden => inject(&mut set_cells, &free_values),
    }
    set_cells.extend(free_cells);
    set_cells
}

/// A cycle shorter than 3 is a pair and would be its own locked set.
fn singles_cycle_len(n: u8, set_size: u8) -> u8 {
    set_size.max(3).min(n - set_size)
}

fn cycle_pair(values: &[u8], i: usize) -> Vec<u8> {
    vec![values[i], values[(i + 1) % values.len()]]
}

/// Cell `i` holds `width` consecutive values starting at `i`. Has no locked set for `1 < width < n`.
fn cyclic_group(n: u8, width: u8) -> Vec<Vec<u8>> {
    (0..n)
        .map(|i| (0..width).map(|t| (i + t) % n + 1).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;
    use crate::cell::Candidates;
    use crate::solver::strategic::strategies::locked_sets::v2::find_locked_set;

    fn assert_bench_cases<Base: SudokuBase>()
    where
        Base::Group<Candidates<Base>>: PartialEq,
    {
        for case in bench_cases::<Base>() {
            let reduced = find_locked_set(&case.candidates_group);
            assert_eq!(
                reduced != case.candidates_group,
                case.has_locked_set,
                "{}:\n{}",
                case.name,
                case.candidates_group
            );
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
    fn test_bench_case_names_unique() {
        let names: Vec<_> = bench_cases::<Base5>()
            .into_iter()
            .map(|case| case.name)
            .collect();
        let mut deduped = names.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(names.len(), deduped.len(), "{names:#?}");
    }
}
