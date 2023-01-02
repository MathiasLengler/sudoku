use sudoku::base::consts::Base3;
use sudoku::Sudoku;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    console_log!("{}", Sudoku::<Base3>::new());

    assert_eq!(1, 1);
}
