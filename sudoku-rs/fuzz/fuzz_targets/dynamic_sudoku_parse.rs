#![no_main]

use std::convert::TryInto;

use libfuzzer_sys::fuzz_target;

use sudoku::{error::Result, DynamicSudoku};

fuzz_target!(|data: String| {
    fuzz(data);
});

fn fuzz(data: String) {
    let _sudoku: Result<DynamicSudoku> = data.as_str().try_into();
}
