use std::convert::TryInto;

use crate::cell::Cell;
use crate::error::Result;
use crate::Sudoku;

pub fn base_2() -> Result<Vec<Sudoku<Cell>>> {
    vec![
        vec![
            vec![0, 3, 4, 0],
            vec![4, 0, 0, 2],
            vec![1, 0, 0, 3],
            vec![0, 2, 1, 0],
        ],
        vec![
            vec![1, 0, 4, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 1, 0, 2],
        ],
        vec![
            vec![0, 0, 1, 0],
            vec![4, 0, 0, 0],
            vec![0, 0, 0, 2],
            vec![0, 3, 0, 0],
        ],
    ]
    .into_iter()
    .map(TryInto::<Sudoku<Cell>>::try_into)
    .collect::<Result<Vec<_>>>()
}

pub fn base_3() -> Result<Vec<Sudoku<Cell>>> {
    vec![
        // 11 Star difficulty
        vec![
            vec![8, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 3, 6, 0, 0, 0, 0, 0],
            vec![0, 7, 0, 0, 9, 0, 2, 0, 0],
            vec![0, 5, 0, 0, 0, 7, 0, 0, 0],
            vec![0, 0, 0, 0, 4, 5, 7, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 3, 0],
            vec![0, 0, 1, 0, 0, 0, 0, 6, 8],
            vec![0, 0, 8, 5, 0, 0, 0, 1, 0],
            vec![0, 9, 0, 0, 0, 0, 4, 0, 0],
        ],
    ]
    .into_iter()
    .map(TryInto::<Sudoku<Cell>>::try_into)
    .collect::<Result<Vec<_>>>()
}
