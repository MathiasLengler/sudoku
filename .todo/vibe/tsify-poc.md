# tsify PoC: surprises

- Root cause: type references are resolved by name only. The derive prints the last path segment as written (`ts_type.rs:191-205`); ts-rs resolves via `<T as TS>::name()` after type checking.
  - Aliases, `use … as X` and renames print a name the Rust source doesn't mean.
  - An undeclared name → `tsc` error (`skipLibCheck: false`). A name that matches another declaration → silently wrong type.
  - The same cause behind the alias/`u64` and `type X = X` bullets below.

- Generic types: a derived `Tsify` puts the bare generic name in signatures (`WorldPosition` instead of `WorldPosition<CellMarker>`), which gives TS2314 ([tsify#76](https://github.com/madonoharu/tsify/issues/76), open).
  - Workaround: the alias declares itself (`tsify::declare`), and a transparent `Wasm*` newtype wraps it (`sudoku-wasm/src/generic_instances.rs`). Used as `Ts<WasmX>` + `.to_rust()?.0`.
  - The `.d.ts` gains a second name per instance (`WasmStrategySet = StrategySet`).
  - `Ts<StrategyMap<bool>>` still compiles. Only `tsc` catches it (TS2314), not rustc.
  - A newtype with the same name as its alias would emit `type X = X`.
  - `DynamicGrid` avoids a newtype: `type_params = "T = DynamicCell"` mirrors the Rust default, so the bare name is valid.
- tsify ignores `serde(try_from, into)` and `serde_repr`: container `tsify(type = …)` string override.
  - `BaseEnum`: the literal list is checked against `BaseEnum::all()` by a test.
  - `DynamicGrid`: `"T[]"` is unchecked.
- World brands: the phantom `T` can't be derived into the brand field, so there's a container `type` override string (unchecked against the Rust fields).
- Rust doc comments are emitted into the `.d.ts` as JSDoc: rationale belongs in `//`.
- `#[tsify::declare]` on aliases ignores container options (`type_alias.rs` uses `TypeGenerationConfig::default()`): `u64` alias → `number`, runtime sends `bigint` (`EvaluatedGridMetric`).
- `large_number_types_as_bigints` (type-gen) only matches literal `u64`/`i64` fields and doesn't see through aliases. No global equivalent of ts-rs's default.
- `SERIALIZATION_CONFIG` is read from the top-level type only; nested types inherit the outer config.
  - → one generic `export_ts` with the crate-wide serializer; `Ts::from_rust` silently diverges.
- `tsify::Error` wraps `JsValue`: it isn't `Send`/`Sync`, so there's no `?` into `anyhow`. Manual `From` into `SudokuWasmError`.
- [tsify#50](https://github.com/madonoharu/tsify/issues/50) (open): defs go missing for cross-crate types → `codegen-units = 1` dev override in the root `Cargo.toml`.
