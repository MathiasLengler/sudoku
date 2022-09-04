#[macro_use]
extern crate criterion;

use std::any::Any;
use std::convert::TryInto;

use criterion::measurement::WallTime;
use criterion::{BatchSize, BenchmarkId};
use criterion::{BenchmarkGroup, Criterion};

use sudoku::base::{consts::*, SudokuBase};
use sudoku::cell::compact::candidates::Candidates;
use sudoku::cell::compact::value::Value;
use sudoku::generator::backtracking::{Generator, Target};
use sudoku::grid::Grid;
use sudoku::position::Position;
use sudoku::samples::{base_2, base_3};
use sudoku::solver::strategic::strategies::GroupReduction;
use sudoku::solver::{backtracking, backtracking_bitset, strategic};

fn cast_grid<Base: SudokuBase + 'static>(any_grid: Box<dyn Any>) -> Grid<Base> {
    *any_grid.downcast().unwrap()
}

fn sample_grid<Base: SudokuBase + 'static>() -> Grid<Base> {
    match Base::BASE {
        2 => cast_grid(Box::new(base_2().into_iter().next().unwrap())),
        3 => cast_grid(Box::new(base_3().into_iter().next().unwrap())),
        _ => panic!("unexpected base"),
    }
}

fn bench_generator_group<Base: SudokuBase>(generator_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;

    for target in &[Target::Minimal, Target::Filled] {
        let parameter_string = format!("Base={} Target={:?}", base, target);
        let generator = Generator::with_target(*target);

        generator_group.bench_with_input(
            BenchmarkId::new("generate", parameter_string),
            &generator,
            |b, generator| {
                b.iter(|| {
                    generator.generate::<Base>();
                })
            },
        );
    }
}

fn bench_solver_group<Base: SudokuBase + 'static>(solver_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;
    let parameter_string = format!("Base={}", base);
    let grid = sample_grid::<Base>();

    solver_group.bench_with_input(
        BenchmarkId::new("backtracking", &parameter_string),
        &grid,
        |b, grid| {
            b.iter_batched(
                || grid.clone(),
                |mut grid| backtracking::Solver::new(&mut grid).next(),
                BatchSize::SmallInput,
            )
        },
    );

    solver_group.bench_with_input(
        BenchmarkId::new("backtracking_bitset", &parameter_string),
        &grid,
        |b, grid| {
            b.iter_batched(
                || grid.clone(),
                |mut grid| backtracking_bitset::Solver::new(&mut grid).try_solve(),
                BatchSize::SmallInput,
            )
        },
    );

    solver_group.bench_with_input(
        BenchmarkId::new("strategic", &parameter_string),
        &grid,
        |b, grid| {
            b.iter_batched(
                || grid.clone(),
                |mut grid| strategic::Solver::new(&mut grid).try_solve(),
                BatchSize::SmallInput,
            )
        },
    );
}

fn bench_grid_group<Base: SudokuBase + 'static>(grid_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::BASE;
    let parameter_string = format!("Base={}", base);
    let grid = sample_grid::<Base>();

    grid_group.bench_with_input(
        BenchmarkId::new("has_duplicate_value", &parameter_string),
        &grid,
        |b, grid| {
            b.iter_batched(
                || grid.row_cells(1),
                |row_cells| Grid::<Base>::has_duplicate_value(row_cells),
                BatchSize::SmallInput,
            )
        },
    );
    grid_group.bench_function(BenchmarkId::new("all_positions", &parameter_string), |b| {
        b.iter(|| {
            Grid::<Base>::all_positions().for_each(|pos| {
                criterion::black_box(pos);
            })
        })
    });

    // Cell iterators
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/row_cells", &parameter_string),
        &grid,
        |b, grid| {
            b.iter(|| {
                grid.row_cells(1).for_each(|cell| {
                    criterion::black_box(cell);
                })
            })
        },
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/all_row_cells", &parameter_string),
        &grid,
        |b, grid| {
            b.iter(|| {
                grid.all_row_cells().for_each(|row| {
                    row.for_each(|cell| {
                        criterion::black_box(cell);
                    });
                })
            })
        },
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/column_cells", &parameter_string),
        &grid,
        |b, grid| {
            b.iter(|| {
                grid.column_cells(1).for_each(|cell| {
                    criterion::black_box(cell);
                })
            })
        },
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/all_column_cells", &parameter_string),
        &grid,
        |b, grid| {
            b.iter(|| {
                grid.all_column_cells().for_each(|column| {
                    column.for_each(|cell| {
                        criterion::black_box(cell);
                    });
                })
            })
        },
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/block_cells", &parameter_string),
        &grid,
        |b, grid| {
            b.iter(|| {
                grid.block_cells(Position { row: 3, column: 3 })
                    .for_each(|cell| {
                        criterion::black_box(cell);
                    })
            })
        },
    );
    grid_group.bench_with_input(
        BenchmarkId::new("iter_cells/all_block_cells", &parameter_string),
        &grid,
        |b, grid| {
            b.iter(|| {
                grid.all_block_cells().for_each(|block| {
                    block.for_each(|cell| {
                        criterion::black_box(cell);
                    });
                })
            })
        },
    );
    grid_group.bench_with_input(
        BenchmarkId::new("direct_candidates", &parameter_string),
        &grid,
        |b, grid| b.iter(|| grid.direct_candidates(Position { column: 1, row: 1 })),
    );
    grid_group.bench_with_input(
        BenchmarkId::new("update_direct_candidates", &parameter_string),
        &grid,
        |b, grid| {
            let mut grid = grid.clone();

            grid.set_all_direct_candidates();

            b.iter_batched(
                || grid.clone(),
                |mut grid: Grid<Base>| {
                    let pos = Position { column: 1, row: 1 };
                    let value = Value::new(2).unwrap().unwrap();
                    grid.get_mut(pos).set_or_toggle_value(value);
                    grid.update_direct_candidates(pos, value);
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
    let candidates_group: Vec<Candidates<U3>> = vec![
        vec![1, 2],
        vec![1, 3],
        vec![2, 3],
        vec![1, 2, 3, 4, 5, 6],
        vec![1, 3, 4],
        vec![2, 3, 4, 5, 6],
    ]
    .into_iter()
    .map(|candidates_data| candidates_data.try_into().unwrap())
    .collect();

    strategy_group.bench_with_input(
        BenchmarkId::new("reduce_candidates_group", "basic"),
        &candidates_group,
        |b, candidates_group| b.iter(|| GroupReduction::reduce_candidates_group(&candidates_group)),
    );
}

fn bench_candidates_group(candidates_group: &mut BenchmarkGroup<WallTime>) {
    candidates_group.bench_function("set", |b| {
        b.iter_batched(
            || {
                (
                    Candidates::<U3>::new(),
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
                    .map(|value| Value::<U3>::try_from(value).unwrap())
                    .collect::<Vec<_>>()
            },
            |candidates_to_set| {
                let mut candidates: u16 = 0;
                for candidate_to_set in candidates_to_set {
                    candidates |= 1 << (candidate_to_set.into_u8() - 1)
                }
                candidates
            },
            BatchSize::SmallInput,
        );
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut generator_group: BenchmarkGroup<WallTime> = c.benchmark_group("Generator");
    bench_generator_group::<U2>(&mut generator_group);
    bench_generator_group::<U3>(&mut generator_group);
    generator_group.finish();

    let mut solver_group = c.benchmark_group("Solver");
    solver_group.sample_size(20);
    bench_solver_group::<U2>(&mut solver_group);
    bench_solver_group::<U3>(&mut solver_group);
    solver_group.finish();

    let mut grid_group = c.benchmark_group("Grid");
    bench_grid_group::<U2>(&mut grid_group);
    bench_grid_group::<U3>(&mut grid_group);
    grid_group.finish();

    let mut strategy_group = c.benchmark_group("Strategies");
    bench_strategy_group(&mut strategy_group);
    strategy_group.finish();

    let mut candidates_group = c.benchmark_group("Candidates");
    bench_candidates_group(&mut candidates_group);
    candidates_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
