# tsify PoC: surprises

- Generic types: a derived `Tsify` puts the bare generic name in signatures (`WorldPosition` instead of `WorldPosition<CellMarker>`), which gives TS2314 ([tsify#76](https://github.com/madonoharu/tsify/issues/76), open).
  - Workaround `tsify_instance!` (`sudoku-rs/src/tsify.rs`): the generic TS decl is written by hand, and nothing checks it against the Rust fields.
  - The alias name is written twice (string literal + Rust alias): `typescript_type` rejects `stringify!`.
  - `stringify!($ty)` breaks for path types (`crate::x::Y`).
- `#[tsify::declare]` on aliases ignores container options (`type_alias.rs` uses `TypeGenerationConfig::default()`): `u64` alias → `number`, runtime sends `bigint` (`EvaluatedGridMetric`).
- `large_number_types_as_bigints` (type-gen) only matches literal `u64`/`i64` fields and doesn't see through aliases. No global equivalent of ts-rs's default.
- `SERIALIZATION_CONFIG` is read from the top-level type only; nested types inherit the outer config.
  - → one generic `export_ts` with the crate-wide serializer; `Ts::from_rust` silently diverges.
- `tsify::Error` wraps `JsValue`: it isn't `Send`/`Sync`, so there's no `?` into `anyhow`. Manual `From` into `SudokuWasmError`.
- [tsify#50](https://github.com/madonoharu/tsify/issues/50) (open): defs go missing for cross-crate types → `codegen-units = 1` dev override in the root `Cargo.toml`.
