use sudoku::base::consts::Base3;
use sudoku::Sudoku;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    let sudoku = Sudoku::<Base3>::new();
    console_log!("{}", sudoku);

    assert_eq!(sudoku.grid().all_cells().count(), 81);
}
