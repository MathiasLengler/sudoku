# Agents Guide

This document provides guidance for AI coding agents working with this repository.

## Project Overview

This is a **Sudoku** application with a touch-optimized web interface. The project consists of multiple packages:

- **sudoku-rs**: Core Rust library for sudoku logic, solvers, and generators
- **sudoku-wasm**: WebAssembly bindings (via `wasm-bindgen`) exposing `sudoku-rs` to JavaScript
- **sudoku-web**: React/TypeScript frontend consuming `sudoku-wasm` in a web worker
- **sudoku-bubblewrap**: Android TWA (Trusted Web Activity) wrapper for Play Store distribution

## Tech Stack

### Rust (sudoku-rs, sudoku-wasm)

- **Edition**: Rust 2024 (Edition 2024)
- **Minimum Rust Version**: 1.92
- **Key Dependencies**:
  - `serde` for serialization
  - `varisat` for SAT-based solving
  - `rayon` for parallel processing (optional)
  - `ts-rs` for TypeScript binding generation
  - `wasm-bindgen` for WebAssembly interop
- **Testing**: Uses `cargo nextest` and `rstest`
- **Linting**: Clippy with pedantic warnings enabled

### TypeScript/React (sudoku-web)

- **Node.js**: 22
- **Build Tool**: Vite
- **UI Framework**: React 19 with MUI (Material UI) 7
- **State Management**: Jotai
- **Type Checking**: TypeScript (via `tsgo`)
- **Linting**: ESLint with `typescript-eslint`, `eslint-plugin-react-hooks`
- **Formatting**: Prettier
- **Testing**: Vitest with Playwright for browser tests

## Repository Structure

```
├── sudoku-rs/           # Core Rust library
│   ├── src/
│   │   ├── base.rs      # Generic base type (Base2-Base5 for different grid sizes)
│   │   ├── cell/        # Cell representation and candidates
│   │   ├── grid/        # Grid structures and parsing
│   │   ├── position/    # Position/coordinate types
│   │   ├── solver/      # Multiple solver implementations
│   │   │   ├── backtracking/    # Backtracking solver
│   │   │   ├── strategic/       # Strategy-based solver
│   │   │   ├── sat/             # SAT-based solver (varisat)
│   │   │   └── introspective/   # Solver with step-by-step explanations
│   │   ├── generator/   # Puzzle generator with pruning
│   │   ├── sudoku/      # High-level Sudoku type with history
│   │   └── world/       # Multi-grid "world" sudoku support
│   ├── bindings/        # Auto-generated TypeScript types (ts-rs)
│   └── benches/         # Criterion benchmarks
│
├── sudoku-wasm/         # WASM bindings
│   ├── src/
│   │   ├── wasm_api/    # wasm-bindgen exposed functions
│   │   └── typescript.rs # TypeScript type helpers
│   └── pkg/             # wasm-pack build output
│
├── sudoku-web/          # React frontend
│   ├── src/
│   │   ├── app/
│   │   │   ├── grid/    # Grid rendering components
│   │   │   ├── state/   # Jotai atoms and state management
│   │   │   │   └── worker/  # Web Worker for WASM calls
│   │   │   ├── actions/ # User action handlers
│   │   │   └── theme/   # MUI theme configuration
│   │   └── main.tsx     # Entry point
│   └── public/          # Static assets
│
└── justfile             # Task runner (just command recipes)
```

## Key Concepts

### SudokuBase Generic Parameter

The core library uses a generic `Base` parameter (Base2, Base3, Base4, Base5) to support different sudoku sizes:

- `Base3` = standard 9×9 sudoku (3×3 blocks)
- `Base4` = 16×16 sudoku (4×4 blocks)

This is defined in [sudoku-rs/src/base.rs](sudoku-rs/src/base.rs).

### Dynamic Types

For WASM interop, dynamic variants exist (e.g., `DynamicGrid`, `DynamicSudoku`) that use runtime base selection instead of compile-time generics.

### Web Worker Architecture

The frontend offloads all WASM/sudoku operations to a web worker using `comlink`. This keeps the UI responsive during generation and solving.

## Development Commands

### Using `just` (Task Runner)

```bash
# List all available recipes
just

# Run all tests and linters (for rust and web)
just lint

# Build WASM (development)
just pack-dev

# Build WASM with file watching
just pack-dev-watch

# Generate TypeScript bindings from Rust
just generate-tsrs-bindings
```

### Frontend (sudoku-web)

```bash
cd sudoku-web

# Install dependencies
npm i

# Start dev server
npm run dev

# Run lints (includes type checking, eslint and prettier)
npm run lint

# Run vitest browser tests
npm test
```

## Code Style & Conventions

### Rust

- Clippy pedantic lints are enabled (see [sudoku-rs/src/lib.rs](sudoku-rs/src/lib.rs))
- Use `#![warn(clippy::pedantic)]` with specific allows for overly strict lints
- Use the crate-specific Result type aliases `crate::error::Result`
  - `sudoku-rs` => Error is `anyhow::Error`
  - `sudoku-wasm`: => Error is `SudokuWasmError`
- Use `#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]` for WASM-exposed types

### TypeScript

- Use `type` over `interface` (enforced by ESLint)
- Unused variables must be prefixed with `_`
- React hooks rules are enforced
- Zod schemas for runtime validation

### Naming Conventions

- Rust: `snake_case` for functions/variables, `PascalCase` for types
- TypeScript: `camelCase` for functions/variables, `PascalCase` for types/components
- Files: `snake_case.rs` for Rust, `camelCase.ts(x)` for TypeScript

## Testing

### Rust

```bash
# Run all tests
just test

# Run specific test
just test test_name

# Run with coverage
just test-cov
```

### TypeScript

```bash
cd sudoku-web

# Run tests
npm test

# Run benchmarks
npm run test:bench

# Type tests
npm run test:types
```

## Building for Production

### WASM Package

```bash
just pack-prod
```

### Web Application

```bash
cd sudoku-web
npm run build
```

### Docker

```bash
cd sudoku-web
npm run docker:build
npm run docker:run
```

## Common Tasks for Agents

### Adding a New Solver Strategy

1. Create strategy in [sudoku-rs/src/solver/strategic/](sudoku-rs/src/solver/strategic/)
2. Add to `StrategyEnum` and `StrategySet`
3. Add tests in the same module

### Modifying WASM API

1. Edit functions in [sudoku-wasm/src/wasm_api/](sudoku-wasm/src/wasm_api/)
2. Run `just pack-dev` to rebuild
3. TypeScript types auto-generate in `sudoku-wasm/pkg/`

### Adding a UI Feature

1. Create component in [sudoku-web/src/app/](sudoku-web/src/app/)
2. Add state atoms in [sudoku-web/src/app/state/](sudoku-web/src/app/state/) if needed
3. For WASM calls, add to worker in [sudoku-web/src/app/state/worker/](sudoku-web/src/app/state/worker/)

### Updating TypeScript Bindings from Rust

```bash
just generate-tsrs-bindings
```

This generates types in [sudoku-rs/bindings/](sudoku-rs/bindings/) from Rust structs decorated with `#[derive(ts_rs::TS)]`.

## Important Files

| File                                | Purpose                                                            |
| ----------------------------------- | ------------------------------------------------------------------ |
| `justfile`                          | All development task recipes                                       |
| `Cargo.toml`                        | Workspace configuration                                            |
| `sudoku-rs/Cargo.toml`              | Core library configuration: feature flags, core dependencies, bins |
| `sudoku-rs/src/lib.rs`              | Core library entry, lint configuration                             |
| `sudoku-rs/src/base.rs`             | Base type definitions (grid sizes)                                 |
| `sudoku-wasm/src/wasm_api/`         | WASM-exposed API                                                   |
| `sudoku-web/vite.config.ts`         | Vite build configuration                                           |
| `sudoku-web/src/app/state/store.ts` | Jotai state setup                                                  |

## Gotchas & Tips

1. **WASM rebuild**: When modifying Rust code, run `just pack-dev` before testing in the browser
2. **Type generation**: After changing Rust types with `ts_rs::TS`, run `just generate-tsrs-bindings`
3. **Multi-shot generator**: Uses rayon for parallel puzzle generation when the `parallel` feature is enabled
