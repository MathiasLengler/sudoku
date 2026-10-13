//! `CandidatesGroup` backed by SIMD vectors, generic over `SudokuBase` and `fearless_simd::Simd`.
//!
//! Every method is `#[inline(always)]`: callers reach it from a `dispatch!`/`#[simd]` context,
//! and a non-inlined call would silently run without the level's target features.
#![expect(clippy::inline_always, reason = "required by fearless_simd")]

use fearless_simd::{Simd, SimdBase, SimdMask};
use num::traits::ConstZero;

use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::grid::group::CandidatesGroup;
use crate::position::Coordinate;

#[derive(Debug, Copy, Clone)]
pub struct SimdCandidatesGroup<Base: SudokuBase, S: Simd> {
    simd: S,
    /// # Invariants
    /// - Lane `i < Base::SIDE_LENGTH` holds a valid `Candidates<Base>`.
    /// - Padding lanes are zero.
    vectors: Base::CandidatesVectors<S>,
}

impl<Base: SudokuBase, S: Simd> SimdCandidatesGroup<Base, S> {
    const LANES: usize = <Base::CandidatesVector<S> as SimdBase<S>>::LEN;

    #[inline(always)]
    pub fn from_fn(simd: S, mut f: impl FnMut(Coordinate<Base>) -> Candidates<Base>) -> Self {
        let vectors = Base::candidates_vectors_from_fn(|vector_index| {
            SimdBase::from_fn(simd, |lane| {
                u8::try_from(vector_index * Self::LANES + lane)
                    .ok()
                    .and_then(|index| Coordinate::new(index).ok())
                    .map_or(Base::CandidatesIntegral::ZERO, |coordinate| {
                        f(coordinate).integral()
                    })
            })
        });
        Self { simd, vectors }
    }

    #[inline(always)]
    pub fn from_group(simd: S, group: &CandidatesGroup<Base>) -> Self {
        Self::from_fn(simd, |coordinate| group.get(coordinate))
    }

    #[inline(always)]
    pub fn to_group(self) -> CandidatesGroup<Base> {
        let mut group = CandidatesGroup::default();
        for (coordinate, candidates) in group.iter_mut_enumerate() {
            *candidates = self.get(coordinate);
        }
        group
    }

    #[inline(always)]
    pub fn get(&self, coordinate: Coordinate<Base>) -> Candidates<Base> {
        let index = usize::from(coordinate.get());
        let vector = self.vectors.as_ref()[index / Self::LANES];
        Candidates::with_integral(vector[index % Self::LANES])
            .expect("lanes to hold valid candidates")
    }

    /// The coordinates whose candidates contain `candidate`.
    #[inline(always)]
    pub fn has(&self, candidate: impl Into<Coordinate<Base>>) -> Candidates<Base> {
        let candidate = Candidates::<Base>::from_iter([candidate]).integral();
        let bitmask = self.vectors.as_ref().iter().enumerate().fold(
            0u64,
            |bitmask, (vector_index, &vector)| {
                let lanes = (vector & candidate)
                    .simd_ne(Base::CandidatesIntegral::ZERO)
                    .to_bitmask();
                bitmask | lanes << (vector_index * Self::LANES)
            },
        );
        // Padding lanes are zero, so only bits below `SIDE_LENGTH` can be set.
        let bitmask = u32::try_from(bitmask).expect("at most 32 coordinates");
        Candidates::with_integral(
            Base::CandidatesIntegral::try_from(bitmask)
                .map_err(Into::into)
                .expect("bitmask to fit `CandidatesIntegral`"),
        )
        .expect("bitmask to hold valid coordinates")
    }

    /// Swaps the roles of coordinates and candidates, see `CandidatesGroup::transpose`.
    #[inline(always)]
    #[must_use]
    pub fn transpose(&self) -> Self {
        Self::from_fn(self.simd, |candidate| self.has(candidate))
    }

    #[inline(always)]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        self.zip_with(other, |vector, other| vector | other)
    }

    #[inline(always)]
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        self.zip_with(other, |vector, other| vector & other)
    }

    #[inline(always)]
    #[must_use]
    pub fn without(self, other: Self) -> Self {
        self.zip_with(other, |vector, other| vector & !other)
    }

    /// `f` must map lanes holding valid candidates (or zero) to valid candidates (or zero).
    #[inline(always)]
    fn zip_with(
        mut self,
        other: Self,
        f: impl Fn(Base::CandidatesVector<S>, Base::CandidatesVector<S>) -> Base::CandidatesVector<S>,
    ) -> Self {
        for (vector, &other) in self.vectors.as_mut().iter_mut().zip(other.vectors.as_ref()) {
            *vector = f(*vector, other);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use fearless_simd::dispatch;
    use rand::prelude::*;

    use super::*;
    use crate::cell::Value;
    use crate::rng::new_crate_rng_with_seed;
    use crate::test_util::{simd_levels, test_max_base5};

    fn random_group<Base: SudokuBase>(rng: &mut impl Rng) -> CandidatesGroup<Base> {
        let mut group = CandidatesGroup::default();
        for candidates in group.iter_mut() {
            *candidates = Value::<Base>::all()
                .filter(|_| rng.random_bool(0.5))
                .collect();
        }
        group
    }

    test_max_base5!({
        let mut rng = new_crate_rng_with_seed(Some(1));
        let levels = simd_levels();
        for _ in 0..200 {
            let a = random_group::<Base>(&mut rng);
            let b = random_group::<Base>(&mut rng);
            let zip = |f: fn(Candidates<Base>, Candidates<Base>) -> Candidates<Base>| {
                CandidatesGroup::<Base>::try_from(
                    a.iter()
                        .zip(b.iter())
                        .map(|(a, b)| f(a, b))
                        .collect::<Vec<_>>(),
                )
                .unwrap()
            };
            let expected = [
                a.clone(),
                a.transpose(),
                zip(Candidates::union),
                zip(Candidates::intersection),
                zip(Candidates::without),
            ];
            for &level in &levels {
                let actual = dispatch!(level, simd => {
                    let simd_a = SimdCandidatesGroup::from_group(simd, &a);
                    let simd_b = SimdCandidatesGroup::from_group(simd, &b);
                    [
                        simd_a.to_group(),
                        simd_a.transpose().to_group(),
                        simd_a.union(simd_b).to_group(),
                        simd_a.intersection(simd_b).to_group(),
                        simd_a.without(simd_b).to_group(),
                    ]
                });
                assert_eq!(actual, expected, "{level:?}\na:\n{a}b:\n{b}");
            }
        }
    });
}
