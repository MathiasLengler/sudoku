# Phase 3b — SAT-Fit Assessment & Strategy Roadmap

Final synthesis. Builds on the human-reviewed `3a-hierarchy-infra.md` (bands, node map,
6 primitives) — does not contradict it. All theory claims trace to `1-theory-spine.md`;
`UNKNOWN` markers preserved, never back-filled. SAT claims grounded in the live
`sat::Solver` API (read from `solver/sat/mod.rs`) and the `optimize-sat-pruning` branch
evidence (`2-sat-pruning.md`). Nothing here is finished strategy code — shapes, signatures,
and prose only, because implementing the strategies is the point of the project.

---

## Deliverable (c) — SAT-fit assessment

### The tool the assessment is grounded in (live `sat::Solver`, verified)

What exists today (`solver/sat/mod.rs`):

- `Solver::new(grid)` = `with_candidates_filter(grid, &())`. `with_candidates_filter(grid, filter)`
  builds a solver by cloning the per-`Base` cached base CNF (`LazyLock` statics, cell/group/
  all-different clauses only) and calling `assume()` **exactly once** inside
  `init_sat_solver_for_grid`: positive unit literals for every filled cell, negative literals
  for every `CandidatesFilter`-denied candidate. The assumption set is fixed at construction.
- Solving: `FallibleSolver::try_solve` (one model) and `IntoIterator → SolverIter`, whose
  `next()` enumerates successive **distinct** solutions by adding the negation of the previous
  model as a clause, then re-solving (model-negation enumeration).
- `step_count()` → `varisat` conflict count (the `GridMetric::SatStepCount` difficulty proxy).
- `Solver<Base>` is `#[derive(Clone)]`.

What does **not** exist (confirmed absent — `2-sat-pruning.md §4`, and by reading the module):

- **No public re-`assume`/re-`solve` on a live `Solver`.** `assume()` is private and called
  once. The only public way to change the denied-candidate set is to build a *fresh* `Solver`
  (new clone of the base CNF + full assumption rebuild), discarding all learned clauses.
- **No unsat-core / failed-assumption read.** Every query in the codebase reads back a plain
  boolean from `solve()`; nothing calls varisat's failed-core API.
- The one place persistent incremental `assume()+solve()` exists is **bespoke inside
  `AmbiguousSolutionChecker`** on the pruning branch — a private generator helper, not a
  reusable public capability, and it relies on a *known target solution* (a single static
  blocking clause) that a strategy operating on a partial grid does not have.

### The core distinction (the "when is SAT the right tool" tutorial)

SAT is a **boolean oracle over the whole grid at once**. It answers "does a complete valid
assignment exist under these constraints?" cheaply and incrementally. It is the right tool
exactly when your question *reduces to satisfiability of the whole grid* and the answer you
need is a yes/no (or "≥2 models"), not a human-legible chain of reasoning.

- **Good (pre-check / partial-logic):** cheap incremental `assume + solve` boolean oracles —
  "is value `v` at `pos` forced / impossible / does the grid still admit a solution / does it
  admit *two*?". These are global satisfiability questions with boolean answers.
- **Bad (not-applicable):** pattern *search* where the pattern **is** the learning content.
  A Y-Wing, a Swordfish, a colouring chain — SAT can tell you *that* candidate `v` is
  eliminable (it's globally forced off), but it cannot produce *the pattern*, and the pattern
  is the entire point. Using SAT here doesn't pre-check the strategy, it *replaces* it, and
  destroys both the learning and the human-legible `Reason`.
- **Borderline (enumeration where conflict/step behaviour matters):** anything that reads
  `step_count()`/model enumeration as a *difficulty signal* rather than a truth value — the
  answer's *cost* is the product, not the answer.

### Per-item classification

| Item (3a node/primitive) | Verdict | One-line reason |
|---|---|---|
| **uniqueness_oracle (P6 plumbing)** | **partial-logic (enabler)** | The reusable incremental assume+solve oracle; the missing public API below. |
| **Unique Rectangles** | **pre-check** | Gate the deadly-pattern deduction on a real uniqueness check instead of the unchecked axiom. |
| **BUG** | **pre-check** | Same: BUG has no meaning without uniqueness; SAT can verify the precondition. |
| **Brute Force (Band 4)** | good fit, already done | Whole-grid solve = the canonical SAT question; already the SAT backend for Base4/5. |
| **Generator score() / GridMetric TODO** | **borderline** | `step_count()` conflict count as difficulty proxy — cost-as-signal, already wired. |
| **fish(n) / X-Wing / Swordfish / Jellyfish** | **not-applicable** | Pattern search; the fish geometry is the learning content. |
| **Y-Wing / XYZ-Wing / W-Wing / Chute Remote Pairs / Rectangle Elimination** | **not-applicable** | Bent-set pattern search; SAT would replace, not pre-check. |
| **Simple Colouring / X-Cycles / Remote Pairs / XY-Chains** | **not-applicable** (see chain caveat) | Chain/graph construction is the deduction; a global "forced?" oracle bypasses it. |
| **candidate_link_graph / group_availability / sees / bivalue (P1–P5)** | **not-applicable** | Structural/geometric primitives; no satisfiability question involved. |

#### uniqueness_oracle (P6) — partial-logic enabler [primary SAT work]

The one piece of genuinely reusable SAT infrastructure. Every "good" use below routes through
it. Reasoning: URs and BUG are the only strategies in the tree whose *logic itself* is a
satisfiability statement ("if both placements admit valid completions, the puzzle isn't
unique"), so a boolean SAT oracle is a faithful — not replacing — check of their precondition.

Grounding: `Solver::with_candidates_filter(grid, filter)` **already** answers the single-query
version ("assume givens + one denial literal, `solve()`, read a boolean") with no new plumbing.
The gap is the *incremental multi-query* version — running many small variations against the
same mostly-fixed grid without paying a fresh clone + assumption rebuild each time.
`AmbiguousSolutionChecker` proves the shape works but is private, generator-scoped, and assumes
a known solution (its single blocking clause). A partial-grid uniqueness check has **no** known
solution, so detecting "≥2 solutions" must fall back to `SolverIter` model-negation (solve,
negate model, solve again) — strictly more work than the generator's shortcut. This is the key
boundary: the generator's cheap trick does not transfer to strategy-time uniqueness checking.

#### Unique Rectangles — pre-check

Both UR branches (`2-unique-rectangles.md`) apply deadly-pattern eliminations resting entirely
on the unchecked single-solution axiom; fed a multi-solution grid they emit logically invalid
eliminations. A SAT pre-check — before firing UR, confirm the grid still has a unique solution
(`has_multiple_solutions()` == false) — is a faithful guard, not a replacement: it validates the
*precondition*, while the human still finds the rectangle and reasons the elimination. Cost is
the concern, not correctness (see cost model): a uniqueness check is a full enumerate-to-second-
solution, so gate it behind a config flag / run it once per grid, not per deduction.

#### BUG — pre-check

Identical shape to UR (`2-misc.md §1`): the BUG+1 placement is only valid because an all-bivalue
grid would be non-unique. SAT can verify the puzzle is uniquely solvable before trusting the
placement. Same faithful-guard framing, same cost caveat.

#### Brute Force — already the canonical good fit

Whole-grid solving *is* the satisfiability question, which is why `introspective::Solver`
already dispatches Base4/5 to SAT. No action; listed so the boundary is visible: this is what a
"SAT implements the whole thing" case looks like, and it's legitimate precisely because there's
no human strategy to learn here — it's the escape hatch.

#### Generator difficulty grading — borderline (the instructive borderline case)

`step_count()` (conflict count) is already used as `GridMetric::SatStepCount`, a difficulty
proxy feeding the `score()`/`GridMetric` TODO (`0-codebase-baseline.md §Strategy ordering`).
This is the borderline archetype: SAT's *answer* (the solution) is irrelevant; its *cost to
reach the answer* is the signal. Legitimate as a difficulty heuristic, but note it measures SAT-
solver effort, which correlates only loosely with human difficulty — the sudokuwiki grading
model (spine §2) is candidate-density + strategy-point based, a different axis. Keep both; don't
conflate conflict count with the sudokuwiki score.

#### The rejected fits (so the boundary is learnable)

- **Fish (X-Wing/Swordfish/Jellyfish):** *not-applicable.* SAT could tell you candidate `v` is
  eliminable at some cell, but the fish rectangle/3×3 pattern is the entire deduction and the
  entire learning goal. A `fish(n)` scan over `GroupCandidateAvailability` is the right tool;
  SAT here is replacement, not pre-check.
- **Bent sets (Y-/XYZ-/W-Wing, Chute Remote Pairs, Rectangle Elimination):** *not-applicable.*
  Same reason — the pincer/hinge geometry is the content. W-Wing and Rectangle Elimination need
  strong/weak links, which come from `candidate_link_graph`, not SAT.
- **Colouring / chains (Simple Colouring, X-Cycles, Remote Pairs, XY-Chains):** *not-applicable*,
  with a caveat worth internalising. SAT can answer "is candidate `v` at `pos` globally forced
  off?" (`is_forced`) — which is a *superset* of every elimination any single-digit chain would
  find. It is tempting to use this as a "partial-logic" accelerator ("SAT says these 5 cells are
  forced, now go find the chain that proves it"). Reject it anyway: it collapses the distinction
  between strategies (everything an AIC/colouring/X-cycle finds is "just forced"), produces no
  `Reason` chain, and removes the difficulty-grading signal (a SAT-forced elimination has no
  strategy score). The chain graph *is* the deliverable.
- **Structural primitives (P1–P5, sees, bivalue):** *not-applicable* — these are geometric/
  bitset indices; there is no satisfiability question to pose.

### Missing public API — signature shapes only

Two layers. Layer 1 generalises the private incremental capability onto the public `Solver`;
Layer 2 is the strategy-facing oracle (P6 home, `solver/sat/`). Shapes only — no bodies.

```rust
// Layer 1 — make the live solver re-queryable (today assumptions are fixed at construction).
impl<Base: SudokuBase> Solver<Base> {
    /// Replace the denied-candidate assumption set on an ALREADY-BUILT solver and keep
    /// varisat's learned clauses (the capability AmbiguousSolutionChecker had to hand-roll).
    fn reassume<Filter: CandidatesFilter<Base>>(&mut self, grid: &Grid<Base>, filter: &Filter);
    fn is_satisfiable(&mut self) -> Result<bool>;
}

// Layer 2 — strategy-facing boolean oracle over a partial grid (P6). Home: solver/sat/.
// Analogous to AmbiguousSolutionChecker but with NO known-solution blocking clause,
// because at strategy time the solution is not yet known.
struct SolutionOracle<Base: SudokuBase> { /* one cloned base CNF + fixed givens */ }

impl<Base: SudokuBase> SolutionOracle<Base> {
    fn new(grid: &Grid<Base>) -> Self;                                   // clone cached base CNF ONCE, assume givens
    fn admits(&mut self, pos: Position<Base>, value: Value<Base>) -> Result<bool>;   // SAT under assignment ⇒ possible
    fn is_forced(&mut self, pos: Position<Base>, value: Value<Base>) -> Result<bool>; // UNSAT under denial ⇒ forced
    fn has_multiple_solutions(&mut self) -> Result<bool>;               // ≥2 distinct models (SolverIter under the hood)
}

// Flagged, NOT designed here — needed only if a SAT-derived Reason payload is ever wanted
// (baseline Reason::Cell TODO). Unplumbed today; unknown whether varisat exposes it cleanly.
impl<Base: SudokuBase> SolutionOracle<Base> {
    fn failing_assumptions(&self) -> impl Iterator<Item = (Position<Base>, Value<Base>)>;
}
```

### Cost model

The whole argument for incremental querying, from the pruning branch's observed behaviour:

- **Fresh solver per query (today's only public path for a changed filter):** each
  `with_candidates_filter` = clone the cached base CNF (O(#clauses)) + rebuild the full
  assumption vector from the grid (O(#filled cells)) + solve from scratch, discarding every
  learned clause. N queries = **N clones + N assumption rebuilds + zero learning carried over.**
- **Incremental (the `SolutionOracle`/`reassume` shape):** one clone at construction, then per
  query only a `Vec<Lit>` rebuild (filter + push, O(#givens)) and `assume() + solve()` on the
  same live solver, so varisat's conflict-clause learning **persists across all queries in the
  pass.** N queries = **1 clone + N cheap assumption swaps + retained learning.** This is exactly
  the delta `AmbiguousSolutionChecker` exploits (one checker per pruning pass, reused across all
  candidate positions).
- **Uniqueness-specific caveat (teaching point):** `has_multiple_solutions` on a *partial* grid
  cannot use the generator's single-blocking-clause shortcut (no known solution), so it costs at
  least two solves plus a growing negation clause via `SolverIter`. Budget it as a once-per-grid
  gate, never a per-candidate inner-loop call.
- **No measurements exist.** The pruning branch's efficiency claims are prose-only doc comments
  with zero benchmark numbers (`2-sat-pruning.md §2,§5`). Any decision to build P6 should carry a
  benchmark, not inherit the branch's unmeasured claims.

### Heuristics for SAT fit (generalise from these repo examples)

1. **Does the question reduce to "is the whole grid still satisfiable (or satisfiable ≥2 ways)?"**
   Yes → SAT (BUG/UR uniqueness gate, Brute Force). No → not SAT.
2. **Is the human-legible pattern the deliverable?** If the `Reason`/chain *is* the product
   (every wing, fish, colouring), SAT replaces rather than pre-checks — reject it, even when it
   would technically find the same eliminations.
3. **Boolean answer vs. cost-as-signal.** A yes/no truth value → oracle use (good). Reading
   `step_count()` as difficulty → borderline; keep it distinct from the sudokuwiki grade.
4. **Many small variations on one fixed grid?** Then the value is *incremental* assume+solve on
   one persistent solver (learned clauses retained) — but that public capability doesn't exist
   yet; only `AmbiguousSolutionChecker`'s private, known-solution version does.
5. **Do you know a target solution in advance?** Generator: yes → cheap single blocking clause.
   Strategy time: no → full model-negation enumeration, materially more expensive. Don't assume
   the generator's SAT cost transfers to strategy-time checks.

---

## Deliverable (d) — Roadmap

### Scoring (all candidates, 1 = weak … 5 = strong)

Criteria: **(1)** infra leverage — how many later strategies it unlocks; **(2)** sudokuwiki
grading/learning progression — natural next difficulty for learn-by-implementing; **(3)**
prerequisite readiness — prereqs already understood/built; **(4)** reference-test-case
availability — usable spine example coordinates for `assert_deductions`; **(5)** generator/
difficulty-grading payoff. Infra rows score (2)/(5) as *timing/grading relevance* of the
primitive itself. `Σ` is an unweighted sum used only to make ranking visible.

| Candidate | (1) leverage | (2) progression | (3) readiness | (4) test cases | (5) generator | Σ |
|---|:--:|:--:|:--:|:--:|:--:|:--:|
| **P2 `Position::sees`** | 5 | 3 | 5 | 5 | 1 | **19** |
| **Y-Wing** | 2 | 5 | 5 | 5 | 3 | **20** |
| **P3 bivalue/count index** | 4 | 3 | 5 | 5 | 1 | **18** |
| **P1 GroupCandidateAvailability (persistent)** | 5 | 3 | 4 | 4 | 3 | **19** |
| **P5 `fish(n)` scan** | 4 | 4 | 3 | 3 | 4 | **18** |
| **Swordfish** | 2 | 4 | 3 | 2 | 4 | **15** |
| **XYZ-Wing** | 1 | 4 | 4 | 4 | 2 | **15** |
| **BUG** | 1 | 3 | 3 | 4 | 4 | **15** |
| **P4 candidate_link_graph** | 5 | 2 | 2 | 2 | 2 | **13** |
| **Chute Remote Pairs** | 1 | 3 | 3 | 3 | 2 | **12** |
| **W-Wing** | 1 | 3 | 2 | 3 | 2 | **11** |
| **Unique Rectangles** | 1 | 3 | 3 | 2 | 3 | **12** |
| **Rectangle Elimination** | 1 | 3 | 2 | 3 | 3 | **12** |
| **Simple Colouring** | 2 | 3 | 2 | 1 | 3 | **11** |
| **Jellyfish** | 1 | 3 | 2 | 1 | 3 | **10** |
| **P6 uniqueness_oracle** | 2 | 2 | 1 | 3 | 2 | **10** |
| **X-Cycles** | 1 | 2 | 1 | 2 | 3 | **9** |
| **Remote Pairs** | 1 | 2 | 2 | 1 | 1 | **7** |
| **XY-Chains** | 1 | 2 | 1 | 1 | 2 | **7** |

Reading the table: the cheap per-cell primitives (P2, P3) and Y-Wing top the ranking because
they combine trivial prerequisites, concrete spine test cases, and (for P2/P3) broad unlock;
P1 ranks alongside them on leverage but carries open design questions. P4 and P6 have high
*structural* leverage but low readiness/test-case scores — they gate the Band-3 tier, which the
spine itself places later and whose worked examples are largely `UNKNOWN`, so they defer. The
tail (Remote Pairs, XY-Chains, X-Cycles) is correctly last: deprecated, unattempted, or
defective-and-off-by-default.

### Chosen steps (in order, respecting 3a dependencies)

#### Step 1 — Per-cell primitives: `Position::sees` (P2) + candidate-count index (P3)

**Build (shapes only).** Two small, orthogonal per-cell primitives that the entire Bent-Set
family reads from:
```rust
impl<Base: SudokuBase> Position<Base> {
    fn sees(self, other: Self) -> bool;   // self-guard baked in: pos.sees(pos) == false
}
impl<Base: SudokuBase> Grid<Base> {
    fn positions_with_candidate_count(&self, count: u8) -> Vec<(Position<Base>, Candidates<Base>)>;
    // bivalue_positions() = count == 2 convenience wrapper on top, for call-site readability
}
```
`sees` composes the existing `to_row`/`to_column`/`to_block`; the count index filters the
existing `Grid::all_candidates_positions()` (`grid/mod.rs:729`).

**Slop reference + defects to avoid.** For `sees`: the 6–8 duplicate helpers across branches
(`shares_house`, `sees_each_other`, `cells_see_each_other`, `sees`/`sees_both`,
`positions_see_each_other`, `shares_group`, `sees_any_of`, plus an inlined unnamed one). *Defect
to avoid:* y_wing/xyz_wing return `true` for `pos == pos` (no self-guard), masked only by
external call-site guards — bake the guard in and match w_wing's `false` semantics
(`2-wings.md §cross-branch`). For the count index: xyz_wing's reuse of `all_candidates_positions`
is the good example; the `find_bivalve_cells` (sic) ad-hoc scans are what it replaces.

**Test against.** Pure unit tests only (no sudokuwiki page needed): `sees` truth table incl.
self-equality; count index against a hand-built grid with known bivalue/trivalue cells.

**Unlocks next.** Y-Wing (Step 2), and every later Bent-Set/UR/chain consumer.

**Why it beats the nearest alternative (starting with P1).** P1 has equal leverage but carries
three unresolved design questions and a heavier refactor; P2/P3 are ~3-line/1-filter primitives
with no design risk, are hard prerequisites for the highest-ranked *strategy* (Y-Wing), and
consolidate a real latent correctness bug (the missing self-guard). Cheapest possible first
move that clears the most-blocked path.

#### Step 2 — Y-Wing (rewritten on the new primitives)

**Build (prose).** Collect bivalue cells (via P3). For each pivot `{X,Y}`, find bivalue cells
sharing a house (via P2) and sharing exactly one candidate; split into "shares X" (leftover Z1)
and "shares Y" (leftover Z2); any pair with `Z1 == Z2 == Z` is a Y-Wing; eliminate Z from every
cell that `sees` both pincers (via P2), excluding the three pattern cells. No persistent index
needed — a per-`execute()` scan is sufficient and correct.

**Slop reference + defects to avoid.** `origin/copilot/implement-y-wing-strategy` — logic is
spine-correct (`2-wings.md`), use it as the algorithm reference. *Defects to avoid:* it is the
weakest-tested of all branches — no `assert_deductions`, no `strategy_snapshot_tests!` macro, no
sudokuwiki example puzzle, ad-hoc `.iter().any(...)` assertions; and it depends on the
un-self-guarded `shares_house`. Write the full dev test template (unit + `assert_deductions_with_grid`
against the spine example + snapshot macro) that the branch omitted.

**Test against.** `https://www.sudokuwiki.org/Y_Wing_Strategy` — spine gives **concrete
coordinates**: pivot E1 `{7,2}`, pincers A1 `{7,1}` / A7 `{1,2}`, eliminate 2 from E7. This is a
ready `assert_deductions_with_grid` case (rare — most Band-2 spine examples are `UNKNOWN`).

**Unlocks next.** XYZ-Wing (same primitives + trivalue hinge; spine example also concrete:
hinge F9, D9 `{1,2}`/F1 `{1,4}`, eliminate 1 from F7) — the natural cheap follow-on, not a
separate roadmap step. (W-Wing waits on the link graph — see deferred.)

**Why it beats the nearest alternative (Chute Remote Pairs, which the spine places *before*
Y-Wing).** Chute Remote Pairs precedes Y-Wing in the spine's teaching order and needs no graph,
but its slop branch built the wrong strategy entirely (`2-misc.md §2`) so there is no correct
reference to port, and it is a niche ~2% technique. Y-Wing is the archetypal Bent-Set to learn,
has a spine-correct reference *and* concrete test coordinates, and is the local root the rest of
the family (`builds_on: [YWing]`) hangs off. Higher on progression, readiness, and test-cases.

#### Step 3 — Persistent `GroupCandidateAvailability` (P1)

**Build (shapes).** The transposed group view already prototyped on
`origin/copilot/add-persistent-group-availability` (`2-infra-group-availability.md`):
```rust
enum Axis { Row, Column }                     // + other(), coordinates_to_pos()
struct GroupCandidateAvailability<Base> { rows, columns, row_major_blocks, column_major_blocks }
struct StrategicGroupAvailability<Base> { candidates: Vec<GroupCandidateAvailability<Base>> }
    fn from_grid(&Grid<Base>) -> Self;  fn get(Value<Base>);  fn iter();
// threaded via additive Strategy::execute_with_availability(self, grid, &avail) (default → execute)
```

**Resolve the Notion task's open questions (address explicitly, per instructions).**
- *Incremental-vs-rebuild:* **start with rebuild-once-per-solver-step, not incremental.** The
  branch wrote the full incremental API (`insert`/`delete`/`delete_candidate`/`set_value`) but
  left it as **dead code — zero call sites** outside its own tests (`2-infra §1,§3`); it is
  never wired into `Deduction::apply`/`Action::apply`. Rebuild-per-step is the proven behaviour-
  preserving refactor (no snapshot churn on the branch). Only wire the incremental path into the
  deduction-application path *and* hold the struct as persistent `SolverPathIter` state if the
  benchmark below shows `from_grid` dominates.
- *Benchmark shape:* the branch's two criterion benches time `from_grid` **in isolation** with no
  comparison and no full-loop measurement — they instrument but don't answer the question
  (`2-infra §3`). Replace with: (a) a criterion bench of a **full `SolverPathIter` solve** over
  sample grids, shared-availability vs. today's per-strategy rebuild; (b) if and only if (a)
  shows construction is hot, a second bench of incremental-update vs. rebuild on a mid-solve
  grid. Decision gate: **do not build the incremental wiring until (a) justifies it.**

**Slop reference + defects to avoid.** The branch is a faithful, bit-identical extraction —
good reference for the *struct*. *Defects to avoid:* shipping the unwired incremental API as
dead code; the block-position resolution seam (no `Axis::coordinates_to_pos` for blocks — block
index→`Position` still needs the separate `BlockSegment` API, `2-infra §Answering`), which the
fish scan and any future link-graph block-links will have to bridge.

**Test against.** Behaviour-preserving refactor — the existing X-Wing / Group Intersection
sudokuwiki reference tests (`x_wing.rs:369-481`, `group_intersection/mod.rs:369-485`) and
snapshots must stay green unchanged; no new puzzle needed.

**Unlocks next.** Swordfish/Jellyfish via `fish(n)` (Step 4), and is the substrate strong/weak
links are derived from for the deferred link-graph tier (a strong link = a group with
`count()==2`, an O(1) popcount).

**Why it beats the nearest alternative (going straight to `fish(n)`/Swordfish).** `fish(n)`
*requires* this substrate (`requires_primitives: [group_candidate_availability, fish_scan]`);
building Swordfish first would mean a fourth copy of the verbatim-copied index. P1 first means
the Swordfish index, the X-Wing index, and Group Intersection all collapse onto one type before
the fish generalisation lands.

#### Step 4 — Generic `fish(n)` scan → Swordfish (+ Jellyfish free) (P5)

**Build (shape).** One scan over P1 parameterised by `n` (`2-fish.md §Fish-family`):
```rust
fn fish_scan<Base>(avail: &GroupCandidateAvailability<Base>, candidate: Value<Base>, axis: Axis, n: usize) -> Deductions<Base>;
// filter lines to 2..=n candidate positions; enumerate C(lines, n); keep unions of size exactly n;
// eliminate the candidate on the covering lines outside the base lines.
```
Swordfish = `n == 3`; Jellyfish = `n == 4` falls out for free. Leave the working X-Wing on its
bespoke pairwise check *or* migrate it to `n == 2` (owner's call — the union-of-2 case reduces
to X-Wing's pairwise-equality check; migrating touches a tested strategy).

**Slop reference + defects to avoid.** `origin/copilot/add-swordfish-strategy` for the n=3 logic
(spine-correct: 2-or-3 per line, union==3). *Defects to avoid:* its index struct + `Axis` enum
are a **byte-for-byte copy** of `x_wing.rs` — do not perpetuate; build on P1 instead. Tests are
100% synthetic loose-invariant assertions ("no elimination inside base rows") that would pass a
wrong axis boundary undetected — write exact `assert_deductions` cases.

**Test against.** `https://www.sudokuwiki.org/Sword_Fish_Strategy` — but the spine flags the
worked example coordinates `UNKNOWN (not captured)`; the owner fetches a concrete 2-2-2 or
3-3-3 example when implementing. Interim: exact-match tests on hand-constructed fish grids, and
verify X-Wing (n=2) reproduces its existing reference-test eliminations through the generic scan.

**Unlocks next.** Jellyfish (`n==4`, essentially free) and completes the Fish sub-tree.

**Why it beats the nearest alternative (candidate_link_graph / Band 3).** P4 has higher
structural leverage but far lower readiness (three incompatible slop builds to reconcile, an
unresolved `Position` vs `(Position,Value)` node decision) and its consumers (Simple Colouring,
X-Cycles) have `UNKNOWN` spine example coordinates and defective/off-by-default slop. `fish(n)`
is a smaller, self-contained generalisation on the P1 substrate just built, with direct
generator-grading payoff (three Tough-tier strategies), and turns three strategies into one
implementation. It is the higher-readiness, better-tested continuation of the tier already in
motion.

#### Step 5 — Housekeeping: mine, then delete the copilot branches

After Steps 1–4 have ported everything worth keeping, delete the mined branches to stop the
duplication from resurfacing. Before deleting, extract the two remaining sound salvage items
not yet consumed above:
- **`add-locked-sets-reasoning`** — hand-port the `Reason::Candidates` population (fills the
  pre-existing `// TODO: produce decuction reasons`; sound, ~95% snapshot churn, `2-misc.md §3`).
  Not a new node — a fix to the existing LockedSets.
- Confirm nothing else is needed from `implement-xyz-wing` (fold into the Y-Wing follow-on).

Then delete: all `copilot/implement-*-wing`, `copilot/add-*-wing`, `copilot/add-swordfish`,
`copilot/*-unique-rectangle*`, `copilot/add-simple-colouring`, `copilot/implement-x-cycles`,
`copilot/implement-new-bug`, `copilot/add-chute-remote-pairs`, `copilot/add-persistent-group-
availability`, `copilot/optimize-sat-pruning`, `copilot/add-locked-sets-reasoning`.

### Deferred (ranked, but below the cut — why they wait)

- **P4 candidate_link_graph → W-Wing, Simple Colouring, X-Cycles, Rectangle Elimination,
  XY-Chains (Band 3).** Highest structural leverage of anything remaining, but lowest readiness:
  three incompatible slop builds to reconcile, the v1-`Position` vs v2-`(Position,Value)` node
  decision to make up front, and consumers whose spine examples are largely `UNKNOWN`. It is the
  natural Step 6 once the Fish/Bent tiers are solid — layer it on P1 (strong link = `count()==2`).
- **P6 uniqueness_oracle → UR / BUG pre-checks.** Orthogonal to the strategy-graph work; needs
  the incremental SAT API (Layer 1/2 above) that doesn't exist and must be built and benchmarked
  first. Deliverable (c) is the design; it lands when the Uniqueness family is the focus, not
  before.
