#[macro_use]
extern crate criterion;

use criterion::Criterion;
use criterion::{BatchSize, BenchmarkId};

use sudoku::cell::Cell;
use sudoku::generator::backtracking::{Generator, Settings, Target};
use sudoku::grid::Grid;
use sudoku::position::Position;
use sudoku::samples::{base_2, base_3};
use sudoku::solver::{backtracking, constraint, strategic};

fn sample_grid(base: usize) -> Grid<Cell> {
    match base {
        2 => base_2().first().unwrap().clone(),
        3 => base_3().first().unwrap().clone(),
        _ => panic!("unexpected base"),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut generator_group = c.benchmark_group("Generator");

    //    generator_group.sample_size(10);

    for base in &[2] {
        for target in &[Target::Minimal, Target::Filled] {
            let parameter_string = format!("Base={} Target={:?}", base, target);
            let generator = Generator::new(Settings {
                base: *base,
                target: Target::Minimal,
            });

            generator_group.bench_with_input(
                BenchmarkId::new("generate", parameter_string),
                &generator,
                |b, generator| {
                    b.iter(|| {
                        generator.generate::<Cell>().unwrap();
                    })
                },
            );
        }
    }
    generator_group.finish();

    let mut solver_group = c.benchmark_group("Solver");
    solver_group.sample_size(20);

    for base in &[2, 3] {
        let parameter_string = format!("Base={}", base);

        let grid = sample_grid(*base);

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
            BenchmarkId::new("constraint", &parameter_string),
            &grid,
            |b, grid| {
                b.iter_batched(
                    || grid.clone(),
                    |mut grid| constraint::Solver::new(&mut grid).try_solve(),
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
    solver_group.finish();

    let mut grid_group = c.benchmark_group("Grid");

    for base in &[2, 3] {
        let parameter_string = format!("Base={}", base);

        let grid = sample_grid(*base);

        grid_group.bench_with_input(
            BenchmarkId::new("has_conflict_at", &parameter_string),
            &grid,
            |b, grid| b.iter(|| grid.has_conflict_at(Position { column: 1, row: 1 })),
        );

        grid_group.bench_with_input(
            BenchmarkId::new("has_duplicate", &parameter_string),
            &grid,
            |b, grid| {
                b.iter_batched(
                    || (&grid, grid.row_cells(1)),
                    |(grid, row_cells)| grid.has_duplicate(row_cells),
                    BatchSize::SmallInput,
                )
            },
        );

        grid_group.bench_with_input(
            BenchmarkId::new("all_positions", &parameter_string),
            &grid,
            |b, grid| b.iter(|| grid.all_positions().for_each(drop)),
        );

        grid_group.bench_with_input(
            BenchmarkId::new("direct_candidates", &parameter_string),
            &grid,
            |b, grid| b.iter(|| grid.direct_candidates(Position { column: 1, row: 1 })),
        );

        grid_group.bench_with_input(
            BenchmarkId::new("update_candidates", &parameter_string),
            &grid,
            |b, grid| {
                let mut grid = grid.clone();

                grid.set_all_direct_candidates();

                b.iter_batched(
                    || grid.clone(),
                    |mut grid| grid.set_or_toggle_value(Position { column: 1, row: 1 }, 2),
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
    grid_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
