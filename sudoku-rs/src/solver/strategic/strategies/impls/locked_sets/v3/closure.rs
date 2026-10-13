use fearless_simd::prelude::*;
use fearless_simd::u16x16;
use fearless_simd_macros::simd;

/// Bitset rows over values: lane `v` holds the values reachable from `v`.
pub(super) type Rows = [u16; 16];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) struct Closure {
    pub(super) reach: Rows,
    pub(super) reached_by: Rows,
}

/// Reflexive-transitive closure of `reach` and its transpose.
///
/// - `reach` must be reflexive on `0..side_length`
/// - lanes and bits `>= side_length` must be zero
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "TODO(vibe): wire into `Matching::new`")
)]
#[simd]
pub(super) fn closure<S: Simd>(simd: S, reach: Rows, side_length: u8) -> Closure {
    let zero = u16x16::splat(simd, 0);
    let mut reach = u16x16::load_array(simd, reach);
    // Warshall; lane k contains k, so it is stable during its own round.
    for k in 0..side_length {
        let has_k = (reach & (1 << k)).simd_ne(zero);
        let reach_k = u16x16::splat(simd, reach[usize::from(k)]);
        reach = has_k.select(reach | reach_k, reach);
    }

    let mut reached_by = Rows::default();
    for (v, reached_by_v) in reached_by
        .iter_mut()
        .enumerate()
        .take(side_length.into())
    {
        #[expect(clippy::cast_possible_truncation, reason = "16 lanes")]
        {
            *reached_by_v = (reach & (1 << v)).simd_ne(zero).to_bitmask() as u16;
        }
    }

    Closure {
        reach: reach.to_array(),
        reached_by,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::new_crate_rng_with_seed;
    use crate::test_util::simd_levels;
    use fearless_simd::dispatch;
    use rand::prelude::*;

    /// The scalar loops `closure` replaces.
    fn closure_scalar(mut reach: Rows, side_length: u8) -> Closure {
        let side_length = usize::from(side_length);
        for k in 0..side_length {
            let reach_k = reach[k];
            for reach_v in &mut reach[..side_length] {
                if *reach_v & (1 << k) != 0 {
                    *reach_v |= reach_k;
                }
            }
        }
        let mut reached_by = Rows::default();
        for (u, reach_u) in reach[..side_length].iter().enumerate() {
            for (v, reached_by_v) in reached_by[..side_length].iter_mut().enumerate() {
                if reach_u & (1 << v) != 0 {
                    *reached_by_v |= 1 << u;
                }
            }
        }
        Closure { reach, reached_by }
    }

    #[test]
    fn test_closure_matches_scalar() {
        let levels = simd_levels();
        let mut rng = new_crate_rng_with_seed(Some(0));
        for side_length in [4, 9, 16] {
            let values_mask = u16::MAX >> (16 - side_length);
            for _ in 0..1000 {
                // Sparse graphs, otherwise nearly every closure is complete.
                let reach: Rows = std::array::from_fn(|v| {
                    if v < usize::from(side_length) {
                        (rng.random::<u16>() & rng.random::<u16>() & rng.random::<u16>()
                            | 1 << v)
                            & values_mask
                    } else {
                        0
                    }
                });
                let expected = closure_scalar(reach, side_length);
                for &level in &levels {
                    let actual = dispatch!(level, simd => closure(simd, reach, side_length));
                    assert_eq!(actual, expected, "{level:?} on {reach:04x?}");
                }
            }
        }
    }
}
