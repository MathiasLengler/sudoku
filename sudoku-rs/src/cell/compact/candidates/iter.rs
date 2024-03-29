use rand::prelude::*;
use std::fmt::{Debug, Display, Formatter};

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;
use crate::rng::CrateRng;

pub trait CandidatesIterator<Base: SudokuBase>:
    ExactSizeIterator<Item = Value<Base>> + Clone + Display
{
    type InitContext: Debug;

    fn from_candidates_with_init_context(
        candidates: Candidates<Base>,
        init_context: &mut Self::InitContext,
    ) -> Self;

    fn peek(&self) -> Option<Value<Base>>;
}

#[derive(Debug, Clone)]
pub struct CandidatesAscIter<Base: SudokuBase> {
    candidates: Candidates<Base>,
}

impl<Base: SudokuBase> Display for CandidatesAscIter<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.candidates, f)
    }
}

impl<Base: SudokuBase> CandidatesIterator<Base> for CandidatesAscIter<Base> {
    type InitContext = ();

    fn from_candidates_with_init_context(
        candidates: Candidates<Base>,
        (): &mut Self::InitContext,
    ) -> Self {
        candidates.into()
    }

    fn peek(&self) -> Option<Value<Base>> {
        self.candidates.first()
    }
}

impl<Base: SudokuBase> Iterator for CandidatesAscIter<Base> {
    type Item = Value<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        let candidate = self.peek();
        if let Some(candidate) = candidate {
            self.candidates.toggle(candidate);
        }
        candidate
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Base: SudokuBase> ExactSizeIterator for CandidatesAscIter<Base> {
    fn len(&self) -> usize {
        self.candidates.count().into()
    }
}

impl<Base: SudokuBase> From<Candidates<Base>> for CandidatesAscIter<Base> {
    fn from(candidates: Candidates<Base>) -> Self {
        Self { candidates }
    }
}

#[derive(Debug, Clone)]
struct CandidatesRandNoPeekIter<Base: SudokuBase> {
    iter: CandidatesAscIter<Base>,
    rng: CrateRng,
}

impl<Base: SudokuBase> Display for CandidatesRandNoPeekIter<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.iter, f)
    }
}

impl<Base: SudokuBase> Iterator for CandidatesRandNoPeekIter<Base> {
    type Item = Value<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        let cloned_iter = self.iter.clone();
        let next = cloned_iter.choose(&mut self.rng);
        if let Some(value) = next {
            self.iter.candidates.toggle(value);
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Base: SudokuBase> ExactSizeIterator for CandidatesRandNoPeekIter<Base> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[derive(Debug, Clone)]
pub struct CandidatesRandIter<Base: SudokuBase> {
    iter: CandidatesRandNoPeekIter<Base>,
    next: Option<Value<Base>>,
}

impl<Base: SudokuBase> Display for CandidatesRandIter<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(next) = self.next {
            Display::fmt(&next, f)?;
        }

        Display::fmt(&self.iter, f)
    }
}

impl<Base: SudokuBase> CandidatesIterator<Base> for CandidatesRandIter<Base> {
    type InitContext = CrateRng;

    fn from_candidates_with_init_context(
        candidates: Candidates<Base>,
        init_rng: &mut Self::InitContext,
    ) -> Self {
        let mut iter = CandidatesRandNoPeekIter {
            iter: candidates.into(),
            rng: CrateRng::from_rng(init_rng).unwrap(),
        };

        Self {
            next: iter.next(),
            iter,
        }
    }

    fn peek(&self) -> Option<Value<Base>> {
        self.next
    }
}

impl<Base: SudokuBase> Iterator for CandidatesRandIter<Base> {
    type Item = Value<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_next = self.next;
        self.next = self.iter.next();
        current_next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Base: SudokuBase> ExactSizeIterator for CandidatesRandIter<Base> {
    fn len(&self) -> usize {
        if self.next.is_some() {
            self.iter.len() + 1
        } else {
            0
        }
    }
}

// TODO: test
