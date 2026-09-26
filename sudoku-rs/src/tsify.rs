/// Declares the TS alias `$alias = $ty` and implements `Tsify` for that one instantiation of a generic type.
///
/// Workaround for tsify#76: a derived `Tsify` on a generic type gives every instantiation the bare generic
/// name in signatures (`WorldPosition` instead of `WorldPosition<CellMarker>`), which TS resolves to `any`.
/// The generic type itself therefore implements no `Tsify` and declares its TS type in a custom section.
macro_rules! tsify_instance {
    // `typescript_type` requires a string literal: `$alias` can't be derived from the Rust alias ident.
    ($alias:literal = $ty:ty) => {
        const _: () = {
            use wasm_bindgen::prelude::*;

            #[wasm_bindgen(typescript_custom_section)]
            const TS_APPEND_CONTENT: &'static str =
                concat!("export type ", $alias, " = ", stringify!($ty), ";");

            #[wasm_bindgen]
            extern "C" {
                #[derive(Clone, Debug)]
                #[wasm_bindgen(typescript_type = $alias)]
                pub type JsType;
            }

            impl ::tsify::Tsify for $ty {
                type JsType = JsType;
                // The custom section const is consumed by `wasm_bindgen`.
                const DECL: &'static str = concat!("export type ", $alias, " = ", stringify!($ty), ";");
            }
        };
    };
}

pub(crate) use tsify_instance;
