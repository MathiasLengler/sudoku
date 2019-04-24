use log::debug;
use sudoku::cell::OptionCell;
use sudoku::Sudoku;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    init();

    append_hello_dom()?;

    // TODO: send to js land
    let sudoku = Sudoku::<OptionCell>::new(3);

    debug!("Hello!");
    debug!("{}", sudoku);

    Ok(())
}

fn append_hello_dom()-> Result<(), JsValue> {
    let window = web_sys::window().expect("should have a Window");
    let document = window.document().expect("should have a Document");

    let p: web_sys::Node = document.create_element("p")?.into();
    p.set_text_content(Some("Hello from Rust, WebAssembly, and Webpack!"));

    let body = document.body().expect("should have a body");
    let body: &web_sys::Node = body.as_ref();
    body.append_child(&p)?;

    Ok(())
}

fn init() {
    use log::Level;
    use std::panic;
    use std::sync::{ONCE_INIT, Once};
    static SET_HOOK: Once = ONCE_INIT;

    #[cfg(feature = "console")]
        SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(Level::Debug);
    });
}
