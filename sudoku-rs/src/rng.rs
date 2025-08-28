use rand::prelude::*;

pub type CrateRng = rand_xoshiro::Xoshiro256StarStar;

pub fn new_crate_rng_with_seed(seed: Option<u64>) -> CrateRng {
    if let Some(seed) = seed {
        CrateRng::seed_from_u64(seed)
    } else {
        CrateRng::from_rng(&mut rand::rng())
    }
}
