#[macro_use]
extern crate criterion;

use std::any::Any;

use criterion::measurement::WallTime;
use criterion::{BatchSize, BenchmarkId};
use criterion::{BenchmarkGroup, Criterion};

use sudoku::base::{consts::*, SudokuBase};
use sudoku::generator::backtracking::{Generator, Target};
use sudoku::grid::Grid;
use sudoku::position::Position;
use sudoku::samples::{base_2, base_3};
use sudoku::solver::{backtracking, constraint, strategic};

fn cast_grid<Base: SudokuBase + 'static>(any_grid: Box<dyn Any>) -> Grid<Base> {
    *any_grid.downcast().unwrap()
}

fn sample_grid<Base: SudokuBase + 'static>() -> Grid<Base> {
    match Base::to_u8() {
        2 => cast_grid(Box::new(base_2().into_iter().next().unwrap())),
        3 => cast_grid(Box::new(base_3().into_iter().next().unwrap())),
        _ => panic!("unexpected base"),
    }
}

fn bench_generator_group<Base: SudokuBase>(generator_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::to_u8();

    for target in &[Target::Minimal, Target::Filled] {
        let parameter_string = format!("Base={} Target={:?}", base, target);
        let generator = Generator::with_target(*target);

        generator_group.bench_with_input(
            BenchmarkId::new("generate", parameter_string),
            &generator,
            |b, generator| {
                b.iter(|| {
                    generator.generate::<Base>().unwrap();
                })
            },
        );
    }
}

fn bench_solver_group<Base: SudokuBase + 'static>(solver_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::to_u8();
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

fn bench_grid_group<Base: SudokuBase + 'static>(grid_group: &mut BenchmarkGroup<WallTime>) {
    let base = Base::to_u8();
    let parameter_string = format!("Base={}", base);
    let grid = sample_grid::<Base>();

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
                |mut grid| {
                    let pos = Position { column: 1, row: 1 };
                    let value = 2;
                    grid.get_mut(pos).set_or_toggle_value(value);
                    grid.update_candidates(pos, value);
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

fn criterion_benchmark(c: &mut Criterion) {
    let mut generator_group: BenchmarkGroup<WallTime> = c.benchmark_group("Generator");
    //    generator_group.sample_size(10);
    bench_generator_group::<U2>(&mut generator_group);
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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
