set windows-shell := ['pwsh.exe', '-CommandWithArgs']
set positional-arguments

# List all recipes
default:
    @just --list

# Run nextest
test test-name="":
    cargo nextest run {{test-name}}

# https://github.com/taiki-e/cargo-llvm-cov
# Run nextest with coverage
test-cov test-name="":
    cargo llvm-cov nextest --lcov --output-path lcov.info -- {{test-name}}

# Run nextest with coverage and generate HTML report
test-cov-html test-name="":
    cargo llvm-cov nextest --branch --html --open -- {{test-name}}

clippy:
    cargo clippy --all-targets --all-features

clippy-ci:
    cargo clippy --all-targets --all-features -- -D warnings

# wasm-pack dev watch
pack-dev-watch: (_pack-watch "pack-dev")
# wasm-pack release watch
pack-prod-watch: (_pack-watch "pack-prod")

_pack-watch recipe:
    watchexec -e rs,toml just {{recipe}}

# wasm-pack dev
pack-dev: (_pack "--dev")
# wasm-pack release
pack-prod: (_pack "--release")

[working-directory: 'sudoku-wasm']
_pack wasm-pack-args:
    wasm-pack build --target web --reference-types --weak-refs {{wasm-pack-args}} . -- -Z build-std=panic_abort,std

# update all globally installed cargo binaries
install-update:
    cargo install-update -a

# Serve vite on tailscale
web-ts-serve:
    tailscale serve 5173
