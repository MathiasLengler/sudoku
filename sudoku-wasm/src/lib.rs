use log::trace;
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
pub use wasm_bindgen_rayon::init_thread_pool;

mod error;
mod typescript;
mod wasm_api;

#[cfg(feature = "console")]
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to track if a panic has occurred.
/// Once set to true, it indicates the WASM module is in an undefined state
/// and should be reset.
#[cfg(feature = "console")]
static HAS_PANICKED: AtomicBool = AtomicBool::new(false);

/// Check if a Rust panic has occurred.
/// Returns true if the WASM module has panicked and should be reset.
#[wasm_bindgen]
pub fn has_panicked() -> bool {
    #[cfg(feature = "console")]
    {
        HAS_PANICKED.load(Ordering::SeqCst)
    }
    #[cfg(not(feature = "console"))]
    {
        false
    }
}

#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console")]
    {
        use log::Level;
        use std::panic;
        use std::sync::Once;

        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            panic::set_hook(Box::new(|panic_info| {
                // Set the panic flag before logging
                HAS_PANICKED.store(true, Ordering::SeqCst);
                // Call the original console_error_panic_hook for logging
                console_error_panic_hook::hook(panic_info);
            }));
            console_log::init_with_level(Level::Info).unwrap();
        });
    }

    trace!("sudoku-wasm initialized");
}
