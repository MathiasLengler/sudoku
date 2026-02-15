set windows-shell := ['pwsh.exe', '-CommandWithArgs']
set positional-arguments := true

# List all recipes
default:
    @just --list

# Run nextest
test *test-args:
    cargo nextest run {{ test-args }}

test-ci *test-args:
    @just test --profile ci {{ test-args }}

# Run all linters
lint: rust-lint web-lint

# Run all rust linters
[parallel]
rust-lint: clippy-ci test pack-prod

# https://github.com/taiki-e/cargo-llvm-cov

# Run nextest with coverage
test-cov *test-args:
    cargo llvm-cov nextest --lcov --output-path lcov.info -- {{ test-args }}

# Run nextest with coverage and generate HTML report
test-cov-html *test-args:
    cargo llvm-cov nextest --branch --html --open -- {{ test-args }}

test-insta *insta-args:
    cargo insta test {{ insta-args }}

test-insta-force: (test-insta "--unreferenced" "auto" "--force-update-snapshots")

test-insta-review *insta-args:
    cargo insta review {{ insta-args }}

# Run clippy
clippy: _clippy

# Run clippy for CI (treat warnings as errors)
clippy-ci: (_clippy "--" "-D" "warnings")

_clippy *clippy-args:
    cargo clippy --all-features {{ clippy-args }}

# wasm-pack dev watch
pack-dev-watch: (_pack-watch "pack-dev")

# wasm-pack release watch
pack-prod-watch: (_pack-watch "pack-prod")

_pack-watch recipe:
    watchexec -e rs,toml just {{ recipe }}

# wasm-pack dev
pack-dev: (_pack "--dev")

# wasm-pack release
pack-prod: (_pack "--release")

[working-directory('sudoku-wasm')]
_pack wasm-pack-args:
    wasm-pack build --target web --reference-types --weak-refs {{ wasm-pack-args }} . -- -Z build-std=panic_abort,std

# update all globally installed cargo binaries
install-update:
    cargo install-update -a

# upgrade all Cargo dependencies to latest
upgrade-latest:
    cargo upgrade -i --verbose
    cargo update

bench *bench-args:
    cargo bench --bench sudoku_benchmark -- {{ bench-args }}

# Serve vite on tailscale
web-ts-serve:
    tailscale serve 5173

# Run CI build/test/lint locally; fork of `.github/workflows/deploy_app.yml`
ci-local:
    just clippy-ci
    just test
    just pack-prod
    cd sudoku-web && npm ci
    cd sudoku-web && npm run lint
    cd sudoku-web && npm run docker:dev

# Generate TypeScript bindings from Rust ("ts_rs" crate)
generate-tsrs-bindings:
    cargo run --bin generate_tsrs_bindings

# Run all web linters
[parallel]
[working-directory('sudoku-web')]
web-lint: web-lint-tsc web-lint-eslint web-lint-prettier

# TypeScript compiler
[working-directory('sudoku-web')]
web-lint-tsc:
    npm run lint:tsc

# ESLint
[working-directory('sudoku-web')]
web-lint-eslint:
    npm run lint:eslint

# Prettier
[working-directory('sudoku-web')]
web-lint-prettier:
    npm run lint:prettier
