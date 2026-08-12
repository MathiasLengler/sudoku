# Phase 0 — Codebase Baseline (`dev` working tree)

Produced by a Sonnet Explore agent during planning. All paths relative to repo root, under `sudoku-rs/src/`.

## Strategic solver core — `solver/strategic/`

**Strategy trait** — `solver/strategic/strategies/mod.rs:18-44`:
`#[enum_dispatch(StrategyEnum)] pub trait Strategy: Debug + Copy + Clone + Eq + Sized` with `name()`, `score() -> StrategyScore` (u64, fixed-point scale 1000), `execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>>`, default `execute_and_apply`.

**`StrategyEnum`** — `strategies/strategy_enum.rs:206-296`: `#[enum_dispatch] #[repr(u8)]` enum wrapping every strategy struct. `all()` (`:229`) gives canonical order = solve-loop try order: NakedSingles, HiddenSingles, NakedPairs, LockedSets, GroupIntersectionBlockToAxis, GroupIntersectionAxisToBlock, GroupIntersectionBoth, XWing, BruteForce. Custom Serialize/Deserialize/FromStr by name. Companions: `StrategyMap<T>` (dense per-strategy map, `:21-119`), `StrategySet = StrategyMap<bool>`, `StrategySelection` trait (`:121-204`), `StrategySet::default_solver_strategies()` (`:175-186`, all except directional GroupIntersection variants).

**Solver loop** — `solver/strategic/mod.rs`: `Solver<Base, GridRef, Strategies>` (`:158`) via `SolverBuilder` (`:79-155`, optional `CandidatesFilter` from backtracking). `execute_strategies_iter` (`:220-238`) tries strategies in order; `try_strategies` (`:248`) → first with progress (used by `SolverPathIter`, `:270-370`, cheapest-first step iterator with `total_score`); `try_all_strategies` (`:241`) → all results per step (`SolverPathAllIter`, `:389-448`, `average_options()`). Loop restarts from the top after any applied deduction (human-style solve path). `SolveStep`/`DynamicSolveStep` for wasm/JSON.

## Deduction model — `solver/strategic/deduction/`

- `Action` (`action.rs:17-20`): `SetValue(Value)` | `DeleteCandidates(Candidates)`; `validate`/`apply`; `Merge` impl (`:120-147`) with conflict rules.
- `Reason` (`reason.rs:17-23`): only `Candidates(Candidates)` today (UI highlighting); TODO for `Reason::Cell` (group highlighting).
- `Deduction<Base>` (`deduction.rs:25-33`): `{ actions: PositionMap<Base, Action>, reasons: PositionMap<Base, Reason> }` — one atomic strategy result. TODO in file: per-deduction cost (naked vs hidden, set size, chain length).
- `Deductions<Base>` (`deductions.rs:23-25`): `BTreeSet<Deduction<Base>>`; `merge_deductions_by_reasons/actions` (`:84-123`); `apply()` merges all and applies atomically.
- `transport.rs`: `TransportDeductions` etc. — serde/ts-rs mirror types for the WASM boundary, round-trip tested.

## Existing strategy implementations — `strategies/impls/`

| Strategy | File | Score | Approach |
|---|---|---|---|
| NakedSingles | `naked_singles.rs` | 1 | cells with `candidates().count()==1` |
| HiddenSingles | `hidden_singles.rs` | 10 | per group `Group<Base, CandidateStats>` histogram |
| NakedPairs | `naked_pairs.rs` | 5 | per group `BTreeMap<Candidates, Vec<Position>>` buckets |
| LockedSets | `locked_sets.rs` + `locked_sets/v2.rs` | 50 | v2 transposes group's `CandidatesGroup` (generic `Group::transpose`, `grid/group/mod.rs:200-210`) to search naked+hidden sets with the same combinatorial code; v1 kept for comparison |
| GroupIntersection{BlockToAxis,AxisToBlock,Both} | `group_intersection/mod.rs` | 100 | pointing pairs / box-line via private `GroupCandidateIndexes` (`:127-175`) |
| XWing | `x_wing.rs` | 200 | own private `GroupCandidateIndexes` (`:221-259`, rows/columns only) |
| BruteForce | `brute_force.rs` | 1_000_000 | delegates to `introspective::Solver`; emits SetValue per remaining cell |

### Transposed candidate structures (duplication already on `dev`)

1. `GroupCandidateIndexes` in `group_intersection/mod.rs:127-175` — per candidate: `rows`, `columns`, `row_major_blocks`, `column_major_blocks` (each `CandidatesGroup<Base>` = which coordinate within the group holds the candidate). Rebuilt every `execute()` via `with_grid()` (full grid scan).
2. `GroupCandidateIndexes` in `x_wing.rs:221-259` — near-identical, smaller (rows/columns only), own `with_grid()` + `.axis(Axis)` accessor. Also rebuilt every call.
3. NOT the same thing: `GroupAvailability<Base, Filter>` in `solver/backtracking/group_availability.rs:15-121` — index → remaining `Candidates` (intersection of the three group axes); used only by the backtracking solver.

Other shared helpers: `Grid::all_group_positions()` (`grid/mod.rs:702-705`, rows+columns+blocks), `Group<Base, T>` / `CandidatesGroup<Base>` (`grid/group/mod.rs`), `PositionMap` (`position/`).

## Test infrastructure

- No rstest in strategy tests (rstest only in generator). Plain `#[test]` + macros from `src/test_util.rs`.
- Handwritten unit tests assert exact `Deductions` against sudokuwiki.org reference puzzles, embedded as literal strings with `// Reference: https://www.sudokuwiki.org/...` comments. Examples: `naked_pairs.rs:180`, `locked_sets.rs:347,454`, `x_wing.rs:369-481`, `group_intersection/mod.rs:369-485`. Assertions via `assert_deductions` / `assert_deductions_with_grid` (`strategies/mod.rs:59-77`).
- Snapshot tests via insta: macro `strategy_snapshot_tests!` (`strategies/mod.rs:79-127`) at the bottom of every strategy file; for each base (2-4) × sample grid renders input, executes+applies, renders output; snapshots in `strategies/impls/snapshots/`. Solver-level solve-path snapshots in `solver/strategic/mod.rs:521-562`.
- Sample grids: `src/samples.rs` + `Base::grid_samples()`.
- **New-strategy test template**: (1) pure-function unit tests, (2) `execute()` tests against 1-2 sudokuwiki example boards via `assert_deductions_with_grid`, (3) `strategy_snapshot_tests!(MyStrategy)`.

## SAT solver — `solver/sat/`

- `Solver<Base>` (`mod.rs:40`) wraps `varisat::Solver`. `cell_variable.rs`: `CellVariable<Base> { pos, value, is_true }` ↔ DIMACS literal bijection.
- Base CNF built once per Base, cached in `LazyLock` statics (`mod.rs:25-32`); grid-specific state expressed as `assume()` literals (grid values positive, `CandidatesFilter` denials negative; `init_sat_solver_for_grid`, `:101-142`). Clauses `:220-325` incl. "each group contains each value" optimization (tdoku reference).
- Usage today: full-grid solving backend only — `introspective::Solver` dispatches Base4/5 to SAT, Base2/3 to backtracking (`introspective/mod.rs:37-51`); uniqueness/generation pruning via `SolverIter` (yields successive distinct solutions by negating previous model); `step_count()` (conflict count) as `GridMetric::SatStepCount` difficulty proxy. **No per-strategy deduction capability.**

## Introspective solver — `solver/introspective/`

Thin dispatcher enum (`Backtracking`/`Sat`/`Done`), picks backend by Base size. Used by the strategic solver's `BruteForce` (`brute_force.rs:19`) as escape hatch. No notion of Deduction/Reason itself.

## Strategy ordering / difficulty grading (open area)

- `strategies/mod.rs:23`: `// TODO: compare current scores with: https://www.sudokuwiki.org/Grading_Puzzles` — scores (1, 10, 5, 50, 100×3, 200, 1e6) are an ad-hoc first pass.
- `deduction.rs:28-33` TODO: per-deduction difficulty (naked vs hidden, set size, chain length).
- `generator/multi_shot/mod.rs:29-63` (`GridMetric`): `StrategyScore` = Σ `strategy.score() * deductions.count()` along solve path (`SolverPathIter::total_score`, `strategic/mod.rs:277-287`); comment at `:50`: "We need to somehow weigh the available strategies by their difficulty".
