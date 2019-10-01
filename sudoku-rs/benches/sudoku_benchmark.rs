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

    generator_group.sample_size(10);

    for base in &[2, 3] {
        for target in &[Target::Minimal, Target::Filled] {}

        let parameter_string = format!("Base: {}", base);
        generator_group.bench_with_input(
            BenchmarkId::new("minimal", parameter_string),
            base,
            |b, base| {
                b.iter(|| {
                    Generator::new(Settings {
                        base: *base,
                        target: Target::Minimal,
                    })
                    .generate::<Cell>()
                    .unwrap();
                })
            },
        );
        generator_group.bench_with_input(
            BenchmarkId::new("filled", parameter_string),
            base,
            |b, base| {
                b.iter(|| {
                    Generator::new(Settings {
                        base: *base,
                        target: Target::Filled,
                    })
                    .generate::<Cell>()
                    .unwrap()
                })
            },
        );
    }
    generator_group.finish();

    //    c.bench(
    //        "Solver",
    //        ParameterizedBenchmark::new(
    //            "backtracking",
    //            |b, base| {
    //                let sudoku = sample_sudoku(*base);
    //
    //                b.iter_batched(
    //                    || sudoku.clone(),
    //                    |mut sudoku| backtracking::Solver::new(&mut sudoku).next(),
    //                    BatchSize::SmallInput,
    //                )
    //            },
    //            vec![2, 3],
    //        )
    //        .with_function("constraint", |b, base| {
    //            let sudoku = sample_sudoku(*base);
    //
    //            b.iter_batched(
    //                || sudoku.clone(),
    //                |mut sudoku| constraint::Solver::new(&mut sudoku).try_solve(),
    //                BatchSize::SmallInput,
    //            )
    //        })
    //        .with_function("strategic", |b, base| {
    //            let sudoku = sample_sudoku(*base);
    //
    //            b.iter_batched(
    //                || sudoku.clone(),
    //                |mut sudoku| strategic::Solver::new(&mut sudoku).try_solve(),
    //                BatchSize::SmallInput,
    //            )
    //        })
    //        .sample_size(20),
    //    );
    //
    //    c.bench(
    //        "Sudoku",
    //        ParameterizedBenchmark::new(
    //            "has_conflict_at",
    //            |b, base| {
    //                let sudoku = sample_sudoku(*base);
    //
    //                b.iter_batched(
    //                    || sudoku.clone(),
    //                    |sudoku| sudoku.has_conflict_at(Position { column: 1, row: 1 }),
    //                    BatchSize::SmallInput,
    //                )
    //            },
    //            vec![2, 3],
    //        )
    //        .with_function("has_duplicate", |b, base| {
    //            let sudoku = sample_sudoku(*base);
    //
    //            b.iter_batched(
    //                || (&sudoku, sudoku.grid().row_cells(1)),
    //                |(sudoku, row_cells)| sudoku.has_duplicate(row_cells),
    //                BatchSize::SmallInput,
    //            )
    //        })
    //        .with_function("all_positions", |b, base| {
    //            let sudoku = sample_sudoku(*base);
    //
    //            b.iter_batched(
    //                || sudoku.clone(),
    //                |sudoku| sudoku.grid().all_positions().for_each(drop),
    //                BatchSize::SmallInput,
    //            )
    //        })
    //        .with_function("direct_candidates", |b, base| {
    //            let sudoku = sample_sudoku(*base);
    //
    //            b.iter_batched(
    //                || sudoku.clone(),
    //                |sudoku| sudoku.direct_candidates(Position { column: 1, row: 1 }),
    //                BatchSize::SmallInput,
    //            )
    //        })
    //        .with_function("update_candidates", |b, base| {
    //            let mut sudoku = sample_sudoku(*base);
    //
    //            sudoku.set_all_direct_candidates();
    //
    //            b.iter_batched(
    //                || sudoku.clone(),
    //                |mut sudoku| sudoku.set_or_toggle_value(Position { column: 1, row: 1 }, 2),
    //                BatchSize::SmallInput,
    //            )
    //        })
    //        .with_function("set_all_direct_candidates", |b, base| {
    //            let sudoku = sample_sudoku(*base);
    //
    //            b.iter_batched(
    //                || sudoku.clone(),
    //                |mut sudoku| sudoku.set_all_direct_candidates(),
    //                BatchSize::SmallInput,
    //            )
    //        }),
    //    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
