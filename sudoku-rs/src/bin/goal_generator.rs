use env_logger::Env;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::goal::GoalGenerator;
use sudoku::generator::{Generator, PruningSettings};
use sudoku::solver::strategic::strategies::StrategyEnum;

type Base = Base3;

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        Env::default().default_filter_or("info,varisat=warn,sudoku::generator::goal=debug"),
    )
    .format_indent(Some(0))
    .init();

    let generator = GoalGenerator::<Base>::new(Generator::with_pruning(PruningSettings {
        strategies: StrategyEnum::default_solver_strategies_no_backtracking(),
        ..Default::default()
    }))?;

    let (total_score, grid) = generator.generate_for_total_strategy_score(10_000);

    println!("Total score: {}\n{grid}", total_score);

    Ok(())
}
