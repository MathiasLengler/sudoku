use axum::response::IntoResponse;
use axum::{routing::get, Json, Router};
use sudoku::base::consts::Base3;
use sudoku::generator::{Generator, GeneratorSettings, GeneratorTarget};
use sudoku::grid::Grid;
use sudoku::solver::strategic::strategies::Backtracking;
use sudoku::transport::TransportSudoku;
use tokio::task::spawn_blocking;

#[tokio::main]
async fn main() {
    env_logger::init();

    // build our application with a single route
    let app = Router::new().route("/sudoku/generate", get(generate_sudoku));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn generate_sudoku() -> impl IntoResponse {
    spawn_blocking(|| {
        let grid: Grid<Base3> = Generator::with_settings(GeneratorSettings {
            target: GeneratorTarget::Minimal {
                set_all_direct_candidates: false,
            },
            strategies: vec![Backtracking.into()],
            seed: Some(0),
        })
        .generate();
        let sudoku = sudoku::Sudoku::with_grid(grid);
        let transport_sudoku = TransportSudoku::from(&sudoku);

        log::info!("{}", sudoku);

        Json(transport_sudoku)
    })
    .await
    .unwrap()
}
