use std::convert::TryInto;

use arbitrary::{Arbitrary, Unstructured};

use sudoku;
use sudoku::{error::Result, DynamicSudoku};

#[cfg(target_os = "linux")]
fn main() {
    afl::fuzz(true, |data| {
        let mut unstructured = Unstructured::new(data);

        if let Ok(s) = String::arbitrary(&mut unstructured) {
            fuzz(s);
        }
    });
}

fn fuzz(data: String) {
    let _: Result<DynamicSudoku> = data.as_str().try_into();
}
