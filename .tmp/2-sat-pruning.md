# Branch: origin/copilot/optimize-sat-pruning

Diff base `b87a1b3a`. Touches exactly `sudoku-rs/src/solver/sat/mod.rs` (+146 lines: new
struct/impl/tests) and `sudoku-rs/src/generator/mod.rs` (wiring, ~15 lines changed). Pure
infrastructure — no new strategy, no `StrategyEnum` entry.

## 1. `AmbiguousSolutionChecker<Base>` — shape and behavior

```
pub struct AmbiguousSolutionChecker<Base: SudokuBase> {
    sat_solver: varisat::Solver<'static>,
    current_value_assumptions: Vec<Lit>,   // one positive unit-literal per still-filled position
    _base: PhantomData<Base>,
}
```

Manual `Debug` (elides the solver internals, shows only the assumption count).

**`new(solution: &Grid<Base>) -> Self`** — `debug_assert!(solution.is_solved())`. Clones the
same per-`Base` cached base CNF that the ordinary `sat::Solver` uses
(`Solver::<Base>::get_initialized_sat_solver()`, the pre-existing `LazyLock<SatSolver>` statics
described in the baseline, one per `Base2..Base5`, containing only the base
cell/group/all-different clauses — no grid-specific content). Adds exactly **one** extra clause:
the disjunction of the negated literal for every `(pos, value)` in `solution` — i.e. a single
"solution-exclusion" / blocking clause that forbids the SAT solver from ever reproducing this
exact full assignment again. Then builds `current_value_assumptions` as one positive unit literal
per filled cell of `solution` and calls `sat_solver.assume(&assumptions)` once.

**`has_ambiguous_solution(&mut self, removed_pos, denied_value) -> Result<bool>`** — on every
call, rebuilds the assumption vector *from scratch* from `current_value_assumptions`: filters out
the literal belonging to `removed_pos`, appends one **negative** literal that denies
`denied_value` specifically at `removed_pos`, calls `sat_solver.assume(&new_assumptions)`, then
`sat_solver.solve()`. Returns `true` (ambiguous) iff SAT. Logic: the only way to satisfy "keep all
other givens, forbid `denied_value` at `removed_pos`, and be different from the known solution
(blocking clause)" is if a genuinely distinct valid completion exists — i.e. this answers
"would removing this given (denying its value) make the puzzle multi-solution?" for one candidate
position at a time, against a fixed already-known target solution.

**`confirm_removal(&mut self, pos)`** — the only mutator of `current_value_assumptions`: drops
`pos`'s literal permanently. Comment states explicitly that a *rejected* candidate needs no update
because assumptions are always rebuilt fresh from the persisted state on the next call — so
speculative (uncommitted) queries can't corrupt state.

### Generator wiring

`generator/mod.rs`: `try_delete_cell_at_pos` → renamed `try_delete_cell_at_pos_with_checker`,
now takes `&mut AmbiguousSolutionChecker<Base>` and returns `Result<Option<Value<Base>>>` (checker
calls are fallible now). Boolean gate unchanged: `can_be_deleted` short-circuits on
`is_solvable_with_strategies` for non-default strategy sets, only falling through to the SAT check
`!checker.has_ambiguous_solution(pos, deleted_value)?` for the default (pure-`BruteForce`) case.
On success calls `checker.confirm_removal(pos)`; on failure, restores the cell value and leaves the
checker untouched (matches the "no action needed" invariant above).

Two checker instances are created per `Generator::generate_pruned_grid` run: one in
`near_minimal_grid`'s caller (with a manual loop calling `checker.confirm_removal` for every
position `near_minimal_grid` already deleted, to resynchronize state) and one for the main prune
loop over `remaining_pruning_positions`. One checker lives for the whole pruning pass over one
solved grid, not per-candidate.

## 2. What it replaced, and the claimed gain

Old path (`try_delete_cell_at_pos`, same file, base commit): for **every candidate position**,
`introspective::Solver::with_filter(grid.clone(), DisallowedCandidateAtPosition{pos, candidate:
deleted_value})` was constructed from scratch, then `.next()` was called (first-solution search).
`introspective::Solver` dispatches by `Base`: Base2/3 → **backtracking** solver, Base4/5 → SAT
(`sat::Solver::new`, itself a fresh clone-of-cached-base-CNF + fresh full-grid assumption set built
from the grid every time). Each call: clones the grid, rebuilds all assumptions from it, and (for
the SAT path) discards the previous call's live solver/learned-clause state entirely; `.next()`
also goes through `SolverIter`'s general "solve → get model → negate previous model → re-solve"
iterator machinery to fetch one solution.

New path: **one** live `varisat::Solver` instance per grid-pruning pass, reused across all
candidate positions via `assume()` + `solve()` only — no per-candidate clone/rebuild of the CNF,
no `SolverIter` model-negation loop (a single static "exclude this known solution" clause replaces
the need to negate a freshly-found model each time, since the target solution is already known up
front here). Claimed gains per the code's own doc comments: "reuses the SAT solver and leverages
incremental solving," "preserving learned clauses." This is also a **backend switch**, not merely
an optimization of an existing SAT path: for Base2/3 (the standard 9×9 case) pruning previously
used backtracking and now always goes through SAT via `AmbiguousSolutionChecker`.

No benchmark numbers appear anywhere in this diff — the efficiency claims are prose-only doc
comments; nothing here measures conflicts/time before vs. after.

## 3. Correctness / quality concerns

- 4 new unit tests, all in `sat::mod::tests::ambiguous_solution_checker`, all on the **same**
  fully-solved `samples::base_2_solved()` grid:
  1. checking any one position on the intact solved grid reports no ambiguity (correct but
     trivial — an intact 16-cell Base2 solution is maximally over-constrained).
  2. `test_non_minimal_grid_has_ambiguous_solution` — name is misleading; it does **not** test a
     non-minimal/pruned grid at all, it repeats the same single-cell-removal check on the intact
     solved grid as test 1.
  3. `confirm_removal` decrements the internal assumption-vector length by 1 — a state-only check.
  4. `test_incremental_removal_finds_ambiguity` loops removing cells one at a time until
     `has_ambiguous_solution` returns true, but the final assertion is
     `found_ambiguity || removed_count < 16`, which is nearly vacuous — it passes whether or not
     ambiguity is ever actually found, as long as not all 16 cells got removed. It does not assert
     the property the test name claims.
- No test compares old-vs-new checker output for parity on the same grid/position (no regression
  test against the `introspective::Solver`-based path it replaces), and no test exercises the
  actual "reject, then check a different position, then accept" interleaving that the generator
  loop performs — the state-reuse-across-mixed-outcomes correctness is unexercised by tests, only
  inspectable by reading the code.
- On inspection the state handling looks correct: since `has_ambiguous_solution` always rebuilds
  its query from `current_value_assumptions` (filter + push), a speculative/rejected query cannot
  leak a stale negative literal into the persisted state; only `confirm_removal` mutates it. This
  matches the explicit code comment but is not test-verified.
- `AmbiguousSolutionChecker` is not `Clone`, and is scoped to one call site consuming one solved
  grid per instance — no reuse across different grids appears or is attempted in this diff.

## 4. Transferable SAT patterns (prose)

- **"Assume X, check UNSAT" as a forced-elimination oracle.** `has_ambiguous_solution`'s shape —
  assume the current givens plus one extra denial literal, call `solve()`, read a boolean — is
  exactly the shape a strategy would need for "is this candidate forced/impossible given the
  current partial grid." The *existing* `sat::Solver` public API (unmodified by this branch)
  already supports this same query shape today: `Solver::with_candidates_filter(grid, filter)`
  builds a solver from the grid's current givens plus a `CandidatesFilter` denying specific
  candidates, and one `solve()`/iteration tells you whether a completion exists. That is a direct,
  already-available substitute for "naive brute-force pattern-existence checking" inside a
  strategy — the owner's stated pre-check use case — with **no new plumbing** required for the
  single-query case. What is missing is the *incremental* multi-query version (next point).
- **Incremental re-solve via assumption swaps on one persistent solver — the most transferable
  piece of infrastructure here, and currently NOT exposed as a general capability.** The base
  `sat::Solver` type builds and immediately fixes its assumption set at construction
  (`Solver::new`/`with_candidates_filter`); there's no public method to re-`assume()` + re-`solve()`
  on an already-built `Solver`. `AmbiguousSolutionChecker` had to reimplement that capability
  itself, bespoke, tied to its own "exclude one known solution" blocking clause. A strategy that
  wants to run many SAT queries against small variations of the same mostly-fixed partial grid
  (e.g., "for each remaining candidate in a cell, would denying it still leave the grid solvable")
  would need the same shape of incremental-assume-and-resolve loop, but generalized: no known
  target solution to exclude, just the grid's current givens as assumptions plus per-query deltas.
  That generalization does not exist in the codebase yet — it is the concrete plumbing gap.
- **Solution enumeration for uniqueness-based strategies (Unique Rectangles, BUG).** The
  pre-existing (unmodified by this branch) `SolverIter` already does general solve → get model →
  negate model → re-solve enumeration of *successive distinct* solutions, which is the natural
  primitive behind any strategy whose logic rests on a uniqueness assumption (URs/BUG reason about
  "if two placements both admit valid completions, the puzzle wouldn't be unique"). The blocking-
  clause trick this branch uses (one static clause excluding a specific known model, rather than
  negating each newly found model in a loop) is a cheaper special case that only works because the
  target solution is already known in advance (true during generation/pruning); a strategy
  reasoning over a partial, not-yet-solved grid would not have that shortcut and would fall back to
  the general `SolverIter` pattern or to ad hoc CandidatesFilter-based single queries.
- **Gap — extracting *which* assumption caused UNSAT.** Neither `AmbiguousSolutionChecker` nor the
  base `sat::Solver` reads varisat's failed-assumption/unsat-core API anywhere in this codebase;
  every query here only reads back a plain boolean from `solve()`. A strategy wanting to explain
  *why* a candidate is eliminated (for a `Reason`/highlighting payload, per the baseline's TODO on
  `Reason::Cell`) would need that core-extraction capability, which is unexercised and unplumbed
  today — confirmed absent from usage in this branch, not confirmed present or absent in varisat
  itself since nothing here touches it either way.

## 5. Cost model (only what's observable in this diff)

- Base CNF is cached per `Base` via pre-existing `LazyLock<SatSolver>` statics (unchanged by this
  branch); `AmbiguousSolutionChecker::new` reuses the exact same
  `Solver::<Base>::get_initialized_sat_solver()` helper the ordinary `sat::Solver` already used, so
  constructing a checker costs one formula clone + one extra blocking clause + one initial
  `assume()` — this happens **once per grid-pruning pass** (two call sites in
  `generate_pruned_grid`: once for `near_minimal_grid`, once for the main prune loop), not once per
  candidate position.
- Per-candidate query (`has_ambiguous_solution`) does not clone or rebuild the CNF or the solver at
  all: it rebuilds only a `Vec<Lit>` (linear filter+push over the current assumption vector, sized
  to the number of still-filled positions) and calls `assume()` + `solve()` on the same live
  `varisat::Solver` instance — so whatever conflict-clause learning varisat's incremental solving
  accumulates persists across all candidate checks within one pass. This is the whole basis of the
  branch's efficiency claim; no measurement of the actual conflict-count or wall-clock delta is
  present in the diff to substantiate the size of the gain.
