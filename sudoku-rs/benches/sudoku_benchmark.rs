#[macro_use]
extern crate criterion;

use criterion::{BatchSize, BenchmarkId};
use criterion::{Criterion, ParameterizedBenchmark};

use sudoku::cell::Cell;
use sudoku::generator::backtracking::{Generator, Settings, Target};
use sudoku::position::Position;
use sudoku::samples::{base_2, base_3};
use sudoku::solver::{backtracking, constraint, strategic};
use sudoku::Sudoku;

fn sample_sudoku(base: usize) -> Sudoku<Cell> {
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

        let sudoku = sample_sudoku(*base);

        solver_group.bench_with_input(
            BenchmarkId::new("backtracking", &parameter_string),
            &sudoku,
            |b, sudoku| {
                b.iter_batched(
                    || sudoku.clone(),
                    |mut sudoku| backtracking::Solver::new(&mut sudoku).next(),
                    BatchSize::SmallInput,
                )
            },
        );

        solver_group.bench_with_input(
            BenchmarkId::new("constraint", &parameter_string),
            &sudoku,
            |b, sudoku| {
                b.iter_batched(
                    || sudoku.clone(),
                    |mut sudoku| constraint::Solver::new(&mut sudoku).try_solve(),
                    BatchSize::SmallInput,
                )
            },
        );

        solver_group.bench_with_input(
            BenchmarkId::new("strategic", &parameter_string),
            &sudoku,
            |b, sudoku| {
                b.iter_batched(
                    || sudoku.clone(),
                    |mut sudoku| strategic::Solver::new(&mut sudoku).try_solve(),
                    BatchSize::SmallInput,
                )
            },
        );
    }
    solver_group.finish();

    let mut sudoku_group = c.benchmark_group("Sudoku");

    for base in &[2, 3] {
        let parameter_string = format!("Base={}", base);

        let sudoku = sample_sudoku(*base);

        sudoku_group.bench_with_input(
            BenchmarkId::new("has_conflict_at", &parameter_string),
            &sudoku,
            |b, sudoku| b.iter(|| sudoku.has_conflict_at(Position { column: 1, row: 1 })),
        );

        sudoku_group.bench_with_input(
            BenchmarkId::new("has_duplicate", &parameter_string),
            &sudoku,
            |b, sudoku| {
                b.iter_batched(
                    || (&sudoku, sudoku.grid().row_cells(1)),
                    |(sudoku, row_cells)| sudoku.has_duplicate(row_cells),
                    BatchSize::SmallInput,
                )
            },
        );

        sudoku_group.bench_with_input(
            BenchmarkId::new("all_positions", &parameter_string),
            &sudoku,
            |b, sudoku| b.iter(|| sudoku.grid().all_positions().for_each(drop)),
        );

        sudoku_group.bench_with_input(
            BenchmarkId::new("direct_candidates", &parameter_string),
            &sudoku,
            |b, sudoku| b.iter(|| sudoku.direct_candidates(Position { column: 1, row: 1 })),
        );

        sudoku_group.bench_with_input(
            BenchmarkId::new("update_candidates", &parameter_string),
            &sudoku,
            |b, sudoku| {
                let mut sudoku = sudoku.clone();

                sudoku.set_all_direct_candidates();

                b.iter_batched(
                    || sudoku.clone(),
                    |mut sudoku| sudoku.set_or_toggle_value(Position { column: 1, row: 1 }, 2),
                    BatchSize::SmallInput,
                )
            },
        );

        sudoku_group.bench_with_input(
            BenchmarkId::new("set_all_direct_candidates", &parameter_string),
            &sudoku,
            |b, sudoku| {
                b.iter_batched(
                    || sudoku.clone(),
                    |mut sudoku| sudoku.set_all_direct_candidates(),
                    BatchSize::SmallInput,
                )
            },
        );
    }
    sudoku_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
