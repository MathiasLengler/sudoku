set shell := ['pwsh.exe', '-CommandWithArgs']
set positional-arguments

default:
    @just --list

test test-name="":
    cargo nextest run {{test-name}}

test-cov test-name="":
    cargo llvm-cov nextest --lcov --output-path lcov.info -- {{test-name}}

test-cov-html test-name="":
    cargo llvm-cov nextest --html --open -- {{test-name}}

pack-dev-watch: (_pack-watch "pack-dev")
pack-prod-watch: (_pack-watch "pack-prod")

_pack-watch recipe:
    watchexec -e rs,toml just {{recipe}}

pack-dev: (_pack "--dev")
pack-prod: (_pack "--release")

[working-directory: 'sudoku-wasm']
_pack wasm-pack-args:
    wasm-pack build --target web --reference-types --weak-refs {{wasm-pack-args}} . -- -Z build-std=panic_abort,std

