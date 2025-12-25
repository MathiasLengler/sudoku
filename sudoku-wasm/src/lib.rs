use log::trace;
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
pub use wasm_bindgen_rayon::init_thread_pool;

mod error;
mod typescript;
mod wasm_api;

#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console")]
    {
        use log::Level;
        use std::panic;
        use std::sync::Once;

        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(Level::Info).unwrap();
        });
    }

    trace!("sudoku-wasm initialized");
}
