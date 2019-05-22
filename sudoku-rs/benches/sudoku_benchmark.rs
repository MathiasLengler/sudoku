#[macro_use]
extern crate criterion;

use criterion::BatchSize;
use criterion::{Criterion, ParameterizedBenchmark};

use sudoku::cell::Cell;
use sudoku::generator::backtracking::{
    BacktrackingGenerator, BacktrackingGeneratorSettings, BacktrackingGeneratorTarget,
};
use sudoku::position::Position;
use sudoku::samples::{base_2, base_3};
use sudoku::solver::backtracking::BacktrackingSolver;
use sudoku::Sudoku;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
        "BacktrackingGenerator",
        ParameterizedBenchmark::new(
            "critical",
            |b, base| {
                b.iter(|| {
                    BacktrackingGenerator::new(BacktrackingGeneratorSettings {
                        base: *base,
                        target: BacktrackingGeneratorTarget::Critical,
                    })
                    .generate::<Cell>()
                    .unwrap()
                })
            },
            vec![2, 3],
        )
        .with_function("filled", |b, base| {
            b.iter(|| {
                BacktrackingGenerator::new(BacktrackingGeneratorSettings {
                    base: *base,
                    target: BacktrackingGeneratorTarget::Filled,
                })
                .generate::<Cell>()
                .unwrap()
            })
        })
        .sample_size(10),
    );

    c.bench(
        "BacktrackingSolver",
        ParameterizedBenchmark::new(
            "next",
            |b, base| {
                let sudoku = match base {
                    2 => base_2().unwrap().first().unwrap().clone(),
                    3 => base_3().unwrap().first().unwrap().clone(),
                    _ => panic!("unexpected base"),
                };

                b.iter_batched(
                    || sudoku.clone(),
                    |sudoku| BacktrackingSolver::new(sudoku).next(),
                    BatchSize::SmallInput,
                )
            },
            vec![2, 3],
        )
        .sample_size(20),
    );

    fn sample_sudoku(base: usize) -> Sudoku<Cell> {
        match base {
            2 => base_2().unwrap().first().unwrap().clone(),
            3 => base_3().unwrap().first().unwrap().clone(),
            _ => panic!("unexpected base"),
        }
    }

    c.bench(
        "Sudoku",
        ParameterizedBenchmark::new(
            "has_conflict_at",
            |b, base| {
                let sudoku = sample_sudoku(*base);

                b.iter_batched(
                    || sudoku.clone(),
                    |sudoku| sudoku.has_conflict_at(Position { column: 1, row: 1 }),
                    BatchSize::SmallInput,
                )
            },
            vec![2, 3],
        )
        .with_function("has_duplicate", |b, base| {
            let sudoku = sample_sudoku(*base);

            b.iter_batched(
                || (&sudoku, sudoku.grid().row_cells(1)),
                |(sudoku, row_cells)| sudoku.has_duplicate(row_cells),
                BatchSize::SmallInput,
            )
        })
        .with_function("all_positions", |b, base| {
            let sudoku = sample_sudoku(*base);

            b.iter_batched(
                || sudoku.clone(),
                |sudoku| sudoku.grid().all_positions().for_each(drop),
                BatchSize::SmallInput,
            )
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
