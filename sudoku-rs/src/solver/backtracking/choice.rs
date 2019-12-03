use std::fmt::{self, Display};

use rand::seq::SliceRandom;

use crate::position::Position;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Choice {
    pos: Position,
    // TODO: use Candidates directly
    candidates: Vec<u8>,
}

impl Choice {
    pub fn new(mut candidates: Vec<u8>, pos: Position, shuffle_candidates: bool) -> Choice {
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

    pub fn selection(&self) -> u8 {
        self.candidates.last().copied().unwrap_or(0)
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}=({:?})", self.pos, self.candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice() {
        let mut choice = Choice::new(vec![1, 2, 4], Position { row: 0, column: 0 }, false);

        assert_eq!(choice.selection(), 1);
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), 2);
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), 4);
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), 0);
        assert_eq!(choice.is_exhausted(), true);
    }
}
