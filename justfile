set shell := ['pwsh.exe', '-CommandWithArgs']
set positional-arguments

default:
    @just --list

test:
    cargo nextest run

pack-dev-watch: (_pack-watch "pack-dev")
pack-prod-watch: (_pack-watch "pack-prod")

_pack-watch recipe:
    watchexec -e rs,toml just {{recipe}}

pack-dev: (_pack "--dev")
pack-prod: (_pack "--release")

[working-directory: 'sudoku-wasm']
_pack wasm-pack-args:
    wasm-pack build --target web {{wasm-pack-args}} . -- -Z build-std=panic_abort,std
