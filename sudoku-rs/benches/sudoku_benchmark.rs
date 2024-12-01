#[macro_use]
extern crate criterion;

use std::any::Any;
use std::hint::black_box;
use std::path::Path;

use criterion::measurement::WallTime;
use criterion::{BatchSize, BenchmarkId, SamplingMode, Throughput};
use criterion::{BenchmarkGroup, Criterion};

use num::Integer;
use sudoku::base::{consts::*, BaseEnum, SudokuBase};
use sudoku::cell::Candidates;
use sudoku::cell::Value;
use sudoku::generator::{Generator, GeneratorSettings, PruningSettings, PruningTarget};
use sudoku::grid::deserialization::read_grids_from_file;
use sudoku::grid::group::CandidatesGroup;
use sudoku::grid::Grid;
use sudoku::position::test_utils::{consume_iter, consume_nested_iter};
use sudoku::position::Coordinate;
use sudoku::position::Position;
use sudoku::rng::{new_crate_rng_from_rng, new_crate_rng_with_seed};
use sudoku::samples::{base_2, base_3, base_4, base_5};
use sudoku::solver::sat;
use sudoku::solver::strategic::strategies::locked_sets::v2::find_locked_set;
use sudoku::solver::strategic::strategies::locked_sets::v2::test_utils::locked_set_test_cases_base_3;
use sudoku::solver::strategic::strategies::{
    GroupIntersectionBoth, HiddenSingles, LockedSets, Strategy, StrategyEnum,
};
use sudoku::solver::{backtracking, introspective, strategic, FallibleSolver, InfallibleSolver};

fn cast_grid<Base: SudokuBase>(any_grid: Box<dyn Any>) -> Grid<Base> {
    *any_grid.downcast().unwrap()
}

fn sample_grid<Base: SudokuBase>() -> Grid<Base> {
    match Base::ENUM {
        BaseEnum::Base2 => cast_grid(Box::new(base_2().into_iter().next().unwrap())),
        BaseEnum::Base3 => cast_grid(Box::new(base_3().into_iter().next().unwrap())),
        BaseEnum::Base4 => cast_grid(Box::new(base_4().into_iter().next().unwrap())),
        BaseEnum::Base5 => cast_grid(Box::new(base_5().into_iter().next().unwrap())),
    }
}

fn bench_generator_group<Base: SudokuBase>(generator_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;

    for (prune_name, prune_settings) in [
        (
            "Backtracking Minimal",
            Some(PruningSettings::<Base> {
                target: PruningTarget::Minimal,
                ..Default::default()
            }),
        ),
        (
            "NoBacktracking Minimal",
            Some(PruningSettings::<Base> {
                target: PruningTarget::Minimal,
                strategies: StrategyEnum::default_solver_strategies_no_backtracking(),
                ..Default::default()
            }),
        ),
        ("None", None),
    ] {
        let parameter_string = format!("Base={} Target={:?}", base, prune_name);

        generator_group.bench_with_input(
            BenchmarkId::new("generate", parameter_string),
            &prune_settings,
            |b, prune_settings| {
                let mut seeds = 0..;

                b.iter(|| {
                    let seed = seeds.next().unwrap();

                    Generator::<Base>::with_settings(GeneratorSettings {
                        prune: prune_settings.clone(),
                        solution: None,
                        seed: Some(seed),
                    })
                    .generate()
                    .unwrap()
                })
            },
        );
    }
}

fn bench_solver_sample_group<Base: SudokuBase>(solver_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;
    let parameter_string = format!("Base={}", base);
    let grid = sample_grid::<Base>();

    // TODO: backtracking parallel

    solver_group.bench_with_input(
        BenchmarkId::new("backtracking", &parameter_string),
        &grid,
        |b, grid| b.iter(|| backtracking::Solver::new(grid).solve().unwrap()),
    );

    solver_group.bench_with_input(
        BenchmarkId::new("backtracking filter empty", &parameter_string),
        &grid,
        |b, grid| {
            b.iter(|| {
                backtracking::Solver::builder(grid)
                    .candidates_filter(Grid::new())
                    .build()
                    .solve()
                    .unwrap()
            })
        },
    );

    solver_group.bench_with_input(
        BenchmarkId::new("backtracking random", &parameter_string),
        &grid,
        |b, grid| {
            let mut rng = new_crate_rng_with_seed(Some(0));
            b.iter(|| {
                backtracking::Solver::builder(grid)
                    .rng(new_crate_rng_from_rng(&mut rng))
                    .build()
                    .solve()
                    .unwrap()
            })
        },
    );

    solver_group.bench_with_input(
        BenchmarkId::new("strategic", &parameter_string),
        &grid,
        |b, grid| {
            b.iter_batched_ref(
                || grid.clone(),
                |grid| strategic::Solver::new(grid).try_solve().unwrap().unwrap(),
                BatchSize::SmallInput,
            )
        },
    );

    solver_group.bench_with_input(
        BenchmarkId::new("introspective", &parameter_string),
        &grid,
        |b, grid| {
            b.iter_batched(
                || grid.clone(),
                |grid| introspective::Solver::new(grid).solve().unwrap(),
                BatchSize::SmallInput,
            )
        },
    );

    solver_group.bench_with_input(
        BenchmarkId::new("sat", &parameter_string),
        &grid,
        |b, grid| b.iter(|| sat::Solver::new(grid).try_solve().unwrap().unwrap()),
    );
}

fn bench_solver_tdoku_group(solver_tdoku_group: &mut BenchmarkGroup<WallTime>) {
    type Base = Base3;

    let tdoku_datasets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("./tests/res/tdoku/");

    let tdoku_datasets = vec![
        "puzzles0_kaggle",
        "puzzles1_unbiased",
        "puzzles2_17_clue",
        "puzzles3_magictour_top1465",
        "puzzles4_forum_hardest_1905",
        "puzzles5_forum_hardest_1905_11+",
        "puzzles6_forum_hardest_1106",
        "puzzles7_serg_benchmark",
        // invalid puzzles with no solutions
        // "puzzles8_gen_puzzles",
    ];

    for tdoku_dataset in tdoku_datasets {
        let path = tdoku_datasets_dir.join(tdoku_dataset);

        let all_grids = read_grids_from_file::<Base>(path).unwrap();
        let grids = &all_grids[..100];

        solver_tdoku_group.throughput(Throughput::Elements(grids.len() as u64));

        solver_tdoku_group.bench_with_input(
            BenchmarkId::new("backtracking", tdoku_dataset),
            grids,
            |b, grids| {
                b.iter(|| {
                    for grid in grids {
                        backtracking::Solver::new(grid).solve().unwrap();
                    }
                })
            },
        );
        solver_tdoku_group.bench_with_input(
            BenchmarkId::new("strategic", tdoku_dataset),
            grids,
            |b, grids| {
                b.iter_batched_ref(
                    || grids.to_vec(),
                    |grids| {
                        for grid in grids {
                            strategic::Solver::new(grid).try_solve().unwrap().unwrap();
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        solver_tdoku_group.bench_with_input(
            BenchmarkId::new("sat", tdoku_dataset),
            grids,
            |b, grids| {
                b.iter(|| {
                    for grid in grids {
                        sat::Solver::new(grid).try_solve().unwrap().unwrap();
                    }
                })
            },
        );
    }
}

fn bench_solver_micro_group<Base: SudokuBase>(solver_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;
    let parameter_string = format!("Base={}", base);
    let grid = sample_grid::<Base>();

    solver_group.bench_with_input(
        BenchmarkId::new(
            "backtracking_bitset_move_best_choice_to_front",
            parameter_string,
        ),
        &grid,
        |b, grid| {
            b.iter_batched_ref(
                || backtracking::Solver::new(grid),
                |solver| solver.move_best_choice_to_front(black_box(1)),
                BatchSize::SmallInput,
            )
        },
    );
}

fn bench_grid_group<Base: SudokuBase>(grid_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;
    let parameter_string = format!("Base={}", base);
    let grid = sample_grid::<Base>();

    grid_group.bench_with_input(
        BenchmarkId::new("has_duplicate_value", &parameter_string),
        &grid,
        |b, grid| {
            b.iter_batched(
                || grid.row_cells(1.try_into().unwrap()),
                |row_cells| Grid::<Base>::has_duplicate_value(row_cells),
                BatchSize::SmallInput,
            )
        },
    );
    grid_group.bench_function(BenchmarkId::new("all_positions", &parameter_string), |b| {
        b.iter(|| consume_iter(Grid::<Base>::all_positions()))
    });

    let coordinate: Coordinate<Base> = 1.try_into().unwrap();

    // Cell iterators
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/row_cells", &parameter_string),
        &grid,
        |b, grid| b.iter(|| consume_iter(grid.row_cells(coordinate))),
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/all_row_cells", &parameter_string),
        &grid,
        |b, grid| b.iter(|| consume_nested_iter(grid.all_row_cells())),
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/column_cells", &parameter_string),
        &grid,
        |b, grid| b.iter(|| consume_iter(grid.column_cells(coordinate))),
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/all_column_cells", &parameter_string),
        &grid,
        |b, grid| b.iter(|| consume_nested_iter(grid.all_column_cells())),
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/block_cells", &parameter_string),
        &grid,
        |b, grid| b.iter(|| consume_iter(grid.block_cells(coordinate))),
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/all_block_cells", &parameter_string),
        &grid,
        |b, grid| b.iter(|| consume_nested_iter(grid.all_block_cells())),
    );

    let pos: Position<Base> = (1, 1).try_into().unwrap();

    grid_group.bench_with_input(
        BenchmarkId::new("direct_candidates", &parameter_string),
        &grid,
        |b, grid| b.iter(|| grid.direct_candidates(black_box(pos))),
    );
    grid_group.bench_with_input(
        BenchmarkId::new("update_direct_candidates_for_new_value", &parameter_string),
        &grid,
        |b, grid| {
            let mut grid = grid.clone();

            grid.set_all_direct_candidates();

            b.iter_batched(
                || (grid.clone(), Value::try_from(2).unwrap()),
                |(mut grid, value)| {
                    grid.get_mut(pos).set_or_toggle_value(value);
                    grid.update_direct_candidates_for_new_value(pos, value);
                },
                BatchSize::SmallInput,
            )
        },
    );
    grid_group.bench_with_input(
        BenchmarkId::new("set_all_direct_candidates", &parameter_string),
        &grid,
        |b, grid| {
            b.iter_batched(
                || grid.clone(),
                |mut grid| grid.set_all_direct_candidates(),
                BatchSize::SmallInput,
            )
        },
    );
}

fn bench_strategy_group(strategy_group: &mut BenchmarkGroup<WallTime>) {
    let mut grid: Grid<Base3> =
        "000000300000071500002400018000009040094618230610700000430897600008140000009000000"
            .parse()
            .unwrap();
    grid.set_all_direct_candidates();
    grid.fix_all_values();
    strategy_group.bench_with_input(
        BenchmarkId::new("HiddenSingles/execute", "sample_grid_hidden_singles"),
        &grid,
        |b, grid| b.iter(|| HiddenSingles.execute(grid).unwrap()),
    );

    for (locked_sets_param_name, candidates_group, _) in locked_set_test_cases_base_3() {
        strategy_group.bench_with_input(
            BenchmarkId::new("LockedSets/find_locked_set", locked_sets_param_name),
            &candidates_group,
            |b, candidates_group| b.iter(|| find_locked_set(candidates_group)),
        );
    }

    let grid: Grid<Base3> =
        "4105300hg281j209i2j081381ag614j20h410hh80318412181h00581033k4109g130342gi0k86s811103m8i4igh0l85805210hla81g20550g12181500h0309090h50120654i0i081032181g10h09054111"
            .parse()
            .unwrap();
    strategy_group.bench_with_input(
        BenchmarkId::new("LockedSets/execute", "sample_grid_hidden_pairs"),
        &grid,
        |b, grid| b.iter(|| LockedSets.execute(grid).unwrap()),
    );

    let grid: Grid<Base3> = "s00905cgdg2103pgc00h03r0ccd85cmcpcece0c0b0g1do036s9sec11c48222g1482c8c0ho421og8o9o1ogc410209sgoi22054gi0o011i6gkiq116q814s0s4ca48kao4s6o4s1003g10610410s0qg081210c".parse().unwrap();
    strategy_group.bench_with_input(
        BenchmarkId::new("GroupIntersection/execute", "sample_grid_pointing_pairs_2"),
        &grid,
        |b, grid| b.iter(|| GroupIntersectionBoth.execute(grid).unwrap()),
    );
}

fn bench_candidates_group(candidates_group: &mut BenchmarkGroup<WallTime>) {
    candidates_group.bench_function("set", |b| {
        b.iter_batched(
            || {
                (
                    Candidates::<Base3>::new(),
                    vec![1, 2, 4, 5, 9]
                        .into_iter()
                        .map(|value| Value::try_from(value).unwrap())
                        .collect::<Vec<_>>(),
                )
            },
            |(mut candidates, candidates_to_set)| {
                for candidate_to_set in candidates_to_set {
                    candidates.set(candidate_to_set, true)
                }
            },
            BatchSize::SmallInput,
        );
    });

    candidates_group.bench_function("set bit twiddling", |b| {
        b.iter_batched(
            || {
                vec![1, 2, 4, 5, 9]
                    .into_iter()
                    .map(|value| Value::<Base3>::try_from(value).unwrap())
                    .collect::<Vec<_>>()
            },
            |candidates_to_set| {
                let mut candidates: u16 = 0;
                for candidate_to_set in candidates_to_set {
                    candidates |= 1 << (candidate_to_set.get() - 1)
                }
                candidates
            },
            BatchSize::SmallInput,
        );
    });

    candidates_group.bench_function(BenchmarkId::new("block_segmentation", "segment"), |b| {
        b.iter_batched(
            || Candidates::<Base3>::with_integral(0b000_101_000),
            |candidates| candidates.block_segmentation(),
            BatchSize::SmallInput,
        );
    });
    candidates_group.bench_function(BenchmarkId::new("block_segmentation", "all"), |b| {
        b.iter_batched(
            Candidates::<Base3>::all,
            |candidates| candidates.block_segmentation(),
            BatchSize::SmallInput,
        );
    });
    candidates_group.bench_function(BenchmarkId::new("block_segmentation", "new"), |b| {
        b.iter_batched(
            Candidates::<Base3>::new,
            |candidates| candidates.block_segmentation(),
            BatchSize::SmallInput,
        );
    });

    candidates_group.bench_function(BenchmarkId::new("combinations", "Base=3 single k=1"), |b| {
        b.iter_batched(
            || {
                (
                    Candidates::<Base3>::with_integral(0b000_010_000),
                    1.try_into().unwrap(),
                )
            },
            |(candidates, k)| consume_iter(candidates.combinations(k)),
            BatchSize::SmallInput,
        );
    });
    for k in Value::<Base3>::all() {
        candidates_group.bench_function(
            BenchmarkId::new("combinations", format!("Base=3 all k={k}")),
            |b| {
                b.iter_batched(
                    || (Candidates::<Base3>::all(), k),
                    |(candidates, k)| consume_iter(candidates.combinations(k)),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn bench_position_group<Base: SudokuBase>(solver_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;
    let parameter_string = format!("Base={}", base);
    let coordinate = Coordinate::<Base>::new(3).unwrap();

    solver_group.bench_function(BenchmarkId::new("iter/all", &parameter_string), |b| {
        b.iter(|| consume_iter(Position::<Base>::all()))
    });
    solver_group.bench_function(BenchmarkId::new("iter/row", &parameter_string), |b| {
        b.iter(|| consume_iter(Position::<Base>::row(black_box(coordinate))))
    });
    solver_group.bench_function(BenchmarkId::new("iter/column", &parameter_string), |b| {
        b.iter(|| consume_iter(Position::<Base>::column(black_box(coordinate))))
    });
    solver_group.bench_function(BenchmarkId::new("iter/block", &parameter_string), |b| {
        b.iter(|| consume_iter(Position::<Base>::block(black_box(coordinate))))
    });
    solver_group.bench_function(BenchmarkId::new("iter/all_rows", &parameter_string), |b| {
        b.iter(|| consume_nested_iter(Position::<Base>::all_rows()))
    });
    solver_group.bench_function(
        BenchmarkId::new("iter/all_columns", &parameter_string),
        |b| b.iter(|| consume_nested_iter(Position::<Base>::all_columns())),
    );
    solver_group.bench_function(
        BenchmarkId::new("iter/all_blocks", &parameter_string),
        |b| b.iter(|| consume_nested_iter(Position::<Base>::all_blocks())),
    );
    solver_group.bench_function(
        BenchmarkId::new("iter/all_groups", &parameter_string),
        |b| b.iter(|| consume_nested_iter(Position::<Base>::all_groups())),
    );
}

fn bench_group_group<Base: SudokuBase>(solver_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;
    let parameter_string = format!("Base={}", base);

    let group: CandidatesGroup<Base> = Candidates::iter_all_lexicographical()
        .take(Base::SIDE_LENGTH.into())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let coordinate = Coordinate::default();

    solver_group.bench_function(
        BenchmarkId::new("from_iter_checked", &parameter_string),
        |b| {
            b.iter_batched(
                || Candidates::iter_all_lexicographical().take(Base::SIDE_LENGTH.into()),
                |iter| CandidatesGroup::<Base>::from_iter_checked(iter),
                BatchSize::SmallInput,
            );
        },
    );

    solver_group.bench_function(BenchmarkId::new("get", &parameter_string), |b| {
        b.iter(|| group.get(black_box(coordinate)));
    });
    solver_group.bench_function(BenchmarkId::new("map", &parameter_string), |b| {
        b.iter_batched(
            || group.clone(),
            |group| group.map(|c| c.count()),
            BatchSize::SmallInput,
        );
    });
    solver_group.bench_function(BenchmarkId::new("reverse", &parameter_string), |b| {
        b.iter_batched(
            || group.clone(),
            |group| group.reverse(),
            BatchSize::SmallInput,
        );
    });
    solver_group.bench_function(BenchmarkId::new("iter", &parameter_string), |b| {
        b.iter(|| consume_iter(group.iter()));
    });
    solver_group.bench_function(BenchmarkId::new("iter_enumerate", &parameter_string), |b| {
        b.iter(|| consume_iter(group.iter_enumerate()));
    });
    let mask = Value::<Base>::all().filter(|v| v.get().is_odd()).collect();
    solver_group.bench_function(
        BenchmarkId::new("iter_index_mask", &parameter_string),
        |b| {
            b.iter(|| consume_iter(group.iter_index_mask(black_box(mask))));
        },
    );
    solver_group.bench_function(BenchmarkId::new("transpose", &parameter_string), |b| {
        b.iter_batched(
            || group.clone(),
            |group| group.transpose(),
            BatchSize::SmallInput,
        );
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut bench_group: BenchmarkGroup<WallTime> = c.benchmark_group("Generator");
    bench_generator_group::<Base2>(&mut bench_group);
    bench_generator_group::<Base3>(&mut bench_group);
    // Too slow
    // bench_generator_group::<Base4>(&mut generator_group);
    bench_group.finish();

    let mut bench_group = c.benchmark_group("SolverSample");
    bench_group.sample_size(20);
    bench_solver_sample_group::<Base2>(&mut bench_group);
    bench_solver_sample_group::<Base3>(&mut bench_group);
    bench_solver_sample_group::<Base4>(&mut bench_group);
    bench_solver_sample_group::<Base5>(&mut bench_group);
    bench_group.finish();

    let mut bench_group = c.benchmark_group("SolverTdoku");
    bench_group.sample_size(10);
    bench_group.sampling_mode(SamplingMode::Flat);
    bench_solver_tdoku_group(&mut bench_group);
    bench_group.finish();

    let mut solver_micro_group = c.benchmark_group("SolverMicro");
    solver_micro_group.sample_size(20);
    bench_solver_micro_group::<Base2>(&mut solver_micro_group);
    bench_solver_micro_group::<Base3>(&mut solver_micro_group);
    solver_micro_group.finish();

    let mut bench_group = c.benchmark_group("Grid");
    bench_grid_group::<Base2>(&mut bench_group);
    bench_grid_group::<Base3>(&mut bench_group);
    bench_group.finish();

    let mut bench_group = c.benchmark_group("Strategies");
    bench_strategy_group(&mut bench_group);
    bench_group.finish();

    let mut bench_group = c.benchmark_group("Candidates");
    bench_candidates_group(&mut bench_group);
    bench_group.finish();

    let mut bench_group = c.benchmark_group("Position");
    bench_position_group::<Base2>(&mut bench_group);
    bench_position_group::<Base3>(&mut bench_group);
    bench_position_group::<Base4>(&mut bench_group);
    bench_group.finish();

    let mut bench_group = c.benchmark_group("Group");
    bench_group_group::<Base2>(&mut bench_group);
    bench_group_group::<Base3>(&mut bench_group);
    bench_group_group::<Base4>(&mut bench_group);
    bench_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
