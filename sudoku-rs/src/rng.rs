use rand::prelude::*;

pub(crate) type CrateRng = rand_xoshiro::Xoshiro256StarStar;

pub(crate) fn new_crate_rng(seed: Option<u64>) -> CrateRng {
    if let Some(seed) = seed {
        CrateRng::seed_from_u64(seed)
    } else {
        CrateRng::from_rng(thread_rng()).unwrap()
    }
}
