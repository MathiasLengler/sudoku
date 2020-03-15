use std::fmt::{self, Display};

use rand::seq::SliceRandom;

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::position::Position;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Choice<Base: SudokuBase> {
    pos: Position,
    candidates: Vec<Value<Base>>,
}

impl<Base: SudokuBase> Choice<Base> {
    pub fn new(
        mut candidates: Vec<Value<Base>>,
        pos: Position,
        shuffle_candidates: bool,
    ) -> Choice<Base> {
        if shuffle_candidates {
            candidates.shuffle(&mut rand::thread_rng())
        } else {
            // Ascending value selection order when selecting values from the end of the vec
            candidates.reverse();
        }

        Self { pos, candidates }
    }

    pub fn set_next(&mut self) {
        let prev_selection = self.candidates.pop();

        debug_assert!(prev_selection.is_some());
    }

    pub fn is_exhausted(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn selection(&self) -> Option<Value<Base>> {
        self.candidates.last().copied()
    }
}

impl<Base: SudokuBase> Display for Choice<Base> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}=({:?})", self.pos, self.candidates)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn test_choice() {
        let mut choice = Choice::new(
            vec![1, 2, 4]
                .into_iter()
                .map(|v| v.try_into().unwrap())
                .collect(),
            Position { row: 0, column: 0 },
            false,
        );

        assert_eq!(choice.selection(), Some(1.try_into().unwrap()));
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), Some(2.try_into().unwrap()));
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), Some(4.try_into().unwrap()));
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), None);
        assert_eq!(choice.is_exhausted(), true);
    }
}
