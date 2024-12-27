set shell := ['pwsh.exe', '-CommandWithArgs']
set positional-arguments

default:
    just --list

test:
    cargo nextest run

[working-directory: 'sudoku-wasm']
pack:
    wasm-pack build --target web . -- -Z build-std=panic_abort,std
