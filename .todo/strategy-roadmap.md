# Strategy Roadmap — Copilot-Slop Analysis Result

Research deliverable for the Notion task "Analyse Copilot Slop" (2026-07-04).
Full evidence trail in `.tmp/` (see appendix); theory grounded in sudokuwiki.org (`.tmp/1-theory-spine.md`, cited per claim there); slop branches used only as evidence about this codebase, never as theory.

**TL;DR:** The next tier is sudokuwiki's "Tough" band, in three families (Fish, Bent Sets, Colouring) over two shared substrate views your `dev` code currently rebuilds per strategy. Recommended next steps: (1) `Position::sees` + candidate-count index, (2) Y-Wing on them, (3) persistent `GroupCandidateAvailability` (rebuild-per-step first, benchmark before wiring incremental), (4) generic `fish(n)` → Swordfish + free Jellyfish, (5) mine locked-sets-reasoning, then delete all copilot branches. SAT fits only as a uniqueness pre-check for UR/BUG — everywhere else it would replace the pattern you want to learn, not pre-check it.

---

## 1. Strategy hierarchy

Full version: `.tmp/3a-hierarchy-infra.md` Part (a).

The evidence resolves into a three-band tree over one substrate:

- **Band 0 — substrate.** Two complementary grid views: the *transposed group view* (candidate → group type → group index → positions; `GroupCandidateIndexes`, already duplicated twice on `dev` in `x_wing.rs` and `group_intersection/mod.rs`, reinvented a third time by the Swordfish branch) and the *per-cell view* (cells filtered by candidate count; ad-hoc in every wing branch). Nothing shares either view today.
- **Band 1 — Basics: complete on `dev`.** NakedSingles → HiddenSingles → NakedPairs → LockedSets → GroupIntersection. Only branch touching it: `add-locked-sets-reasoning`, a sound Reason-payload fix worth hand-porting (not a new strategy).
- **Band 2 — Tough: the natural next tier.** Three families:
  - **Fish**: X-Wing (`dev`, n=2) → Swordfish (n=3) → Jellyfish (n=4). Sudokuwiki: "All Swordfishes will break down into X-Wings" — one pattern parameterized by n.
  - **Bent Sets**: Y-Wing (local root) → W-Wing / XYZ-Wing; also Chute Remote Pairs (never actually built — the branch produced deprecated generic Remote Pairs instead) and Rectangle Elimination (also never built — that branch produced Unique Rectangles).
  - **Colouring / single-digit chains**: Simple Colouring → X-Cycles; the entry to Band 3.
- **Band 3 — Uniqueness + Diabolical.** Unique Rectangles + BUG (both rest on the unchecked single-solution axiom — the SAT hook), then X-Cycles / XY-Chains (need the link graph with `(Position, Value)` nodes).
- **Band 4 — Backend.** BruteForce via introspective SAT/backtracking; escape hatch, not a human strategy.

Key structural fact: a **strong link is a group whose candidate-position count is 2** — an O(1) popcount on the transposed group view. The chain/link graph (Band 2 Colouring, Band 3) therefore *layers on* GroupCandidateAvailability rather than being independent infrastructure.

### Machine-readable map

`grade` = sudokuwiki tier/points where the fetched pages state one, else `UNKNOWN`. Statuses: `implemented` (on dev) / `slop-correct` / `slop-defective` / `slop-mislabeled` / `absent`.

```yaml
- strategy: NakedSingles
  family: Basic
  grade: F                       # grading table (density factor)
  status: implemented
  requires_primitives: [single_candidate_cell]
  builds_on: []
  example_urls: [https://www.sudokuwiki.org/Getting_Started]
  slop_notes: ""

- strategy: HiddenSingles
  family: Basic
  grade: F×2
  status: implemented
  requires_primitives: [group_candidate_availability]
  builds_on: [NakedSingles]
  example_urls: [https://www.sudokuwiki.org/Getting_Started]   # no standalone page (404)
  slop_notes: ""

- strategy: NakedPairs
  family: Basic
  grade: 5×F                     # Naked Pair; Triples/Quads UNKNOWN
  status: implemented
  requires_primitives: [bivalue_cell]
  builds_on: [NakedSingles]
  example_urls: [https://www.sudokuwiki.org/Naked_Candidates]
  slop_notes: ""

- strategy: LockedSets
  family: Basic
  grade: UNKNOWN
  status: implemented
  requires_primitives: [group_candidate_availability, nvalue_cell]
  builds_on: [NakedPairs]
  example_urls: [https://www.sudokuwiki.org/Hidden_Candidates, https://www.sudokuwiki.org/Naked_Candidates]
  slop_notes: "add-locked-sets-reasoning branch fills the empty Reason::Candidates payload (sound, ~95% snapshot churn, worth hand-porting); not a new node"

- strategy: GroupIntersection
  family: Basic
  grade: UNKNOWN
  status: implemented
  requires_primitives: [group_candidate_availability]
  builds_on: [NakedPairs]
  example_urls: [https://www.sudokuwiki.org/Intersection_Removal]
  slop_notes: "on dev owns one of the two duplicate GroupCandidateIndexes (all 4 axes)"

- strategy: ChuteRemotePairs
  family: BentSets
  grade: UNKNOWN                 # exemplars 6.5-8.5 (pre-Oct-2025 scale)
  status: absent
  requires_primitives: [bivalue_cell, chute_analysis, sees_predicate]
  builds_on: [XWing]             # spine: "should follow X-Wing instruction and precedes Y-Wings"
  example_urls: [https://www.sudokuwiki.org/Chute_Remote_Pairs]
  slop_notes: "add-chute-remote-pairs branch built generic Remote Pairs instead; the real chute technique was never written — needs bivalue index + chute check, NO graph machinery"

- strategy: XWing
  family: Fish
  grade: 30                      # fixed points; own page "Tough"
  status: implemented
  requires_primitives: [group_candidate_availability, strong_link, fish_scan]
  builds_on: [GroupIntersection]
  example_urls: [https://www.sudokuwiki.org/X_Wing_Strategy]
  slop_notes: "on dev owns the second duplicate GroupCandidateIndexes (rows/columns only), copy-pasted verbatim by the Swordfish branch"

- strategy: YWing
  family: BentSets
  grade: Tough                   # exemplars 6.1-7.6
  status: slop-correct
  requires_primitives: [bivalue_cell, sees_predicate]
  builds_on: [XWing]
  example_urls: [https://www.sudokuwiki.org/Y_Wing_Strategy]
  slop_notes: "shares_house returns true for pos==pos (no self-guard); weakest tests of the 3 wings — no assert_deductions, no snapshot macro, no sudokuwiki example puzzle"

- strategy: WWing
  family: BentSets
  grade: Tough                   # "positioned after Y-Wings"
  status: slop-correct
  requires_primitives: [bivalue_cell, strong_link, sees_predicate]
  builds_on: [YWing]
  example_urls: [https://www.sudokuwiki.org/W_Wing_Strategy]
  slop_notes: "single-W-Wing case only (remote-pair-chain & split-double variants absent); own StrongLinksMap can emit a pair twice; no snapshot macro, no sudokuwiki example"

- strategy: XyzWing
  family: BentSets
  grade: UNKNOWN
  status: slop-correct
  requires_primitives: [bivalue_cell, nvalue_cell, sees_predicate]
  builds_on: [YWing]             # spine: "extends the Y-Wing pattern by adding an extra candidate to the hinge"
  example_urls: [https://www.sudokuwiki.org/XYZ_Wing]
  slop_notes: "does NOT enforce a 'same chute' constraint the spine's prose mentions — UNCERTAIN whether that is a real precondition; has snapshot macro but no sudokuwiki-example assert"

- strategy: RectangleElimination
  family: Fish                   # index: Chaining; own page "Tough"
  grade: Tough                   # "between Y-Wing and Swordfish"
  status: absent
  requires_primitives: [group_candidate_availability, strong_link, weak_link, chute_analysis, sees_predicate]
  builds_on: [YWing]
  example_urls: [https://www.sudokuwiki.org/Rectangle_Elimination]
  slop_notes: "the branch NAMED add-rectangle-elimination actually built Unique Rectangles; the real hinge/strong-link technique was never written"

- strategy: Swordfish
  family: Fish
  grade: Tough                   # exemplars 6.3-7.9; points UNKNOWN
  status: slop-correct
  requires_primitives: [group_candidate_availability, fish_scan]
  builds_on: [XWing]             # spine: "All Swordfishes will break down into X-Wings"
  example_urls: [https://www.sudokuwiki.org/Sword_Fish_Strategy]   # worked-example coords UNKNOWN (not captured)
  slop_notes: "logic sound (2-or-3 per line, union==3); index struct + Axis enum VERBATIM copy of x_wing.rs; tests 100% synthetic loose-invariant — a wrong axis boundary would pass"

- strategy: Jellyfish
  family: Fish
  grade: UNKNOWN                 # page not fetched
  status: absent
  requires_primitives: [group_candidate_availability, fish_scan]
  builds_on: [Swordfish]
  example_urls: []
  slop_notes: "unattempted; drops out for free as fish_scan(n=4)"

- strategy: SimpleColouring
  family: Colouring
  grade: Tough
  status: slop-correct
  requires_primitives: [strong_link, candidate_link_graph, sees_predicate]
  builds_on: [XWing]
  example_urls: [https://www.sudokuwiki.org/Simple_Colouring]      # worked example UNKNOWN (not captured)
  slop_notes: "Rules 2 & 4 correct; Rule 7 ('one colour empties a unit') OMITTED; no snapshot macro, no reference puzzle"

- strategy: UniqueRectangles
  family: Uniqueness
  grade: UNKNOWN
  status: slop-defective
  requires_primitives: [bivalue_cell, uniqueness_oracle, sees_predicate]
  builds_on: []
  example_urls: [https://www.sudokuwiki.org/Unique_Rectangles]     # several type coords UNKNOWN
  slop_notes: "TWO duplicate branches (implement-unique-rectangles = Types 1/2/4; add-rectangle-elimination = Types 1/2, MISNAMED). The misnamed branch is the better skeleton (idiomatic Deduction helpers, snapshot macro, sounder Type 2 scope). The other's Type 4 floor-side branch is likely UNSOUND and untested. Both rest on unchecked uniqueness axiom"

- strategy: BUG
  family: Uniqueness
  grade: 3.2-5.3                  # puzzles requiring it
  status: slop-correct
  requires_primitives: [bivalue_cell, group_candidate_availability, uniqueness_oracle]
  builds_on: []                   # spine: "every instance of BUG can be solved by an XY-Chain"
  example_urls: [https://www.sudokuwiki.org/BUG]
  slop_notes: "correct BUG+1 rule; no ambiguity guard if >1 candidate has count==3; no reference-puzzle test, no snapshot macro; relies on uniqueness axiom"

- strategy: RemotePairs
  family: Chaining               # listed under Chaining AND Deprecated
  grade: UNKNOWN
  status: slop-mislabeled
  requires_primitives: [bivalue_cell, candidate_link_graph, sees_predicate]
  builds_on: [SimpleColouring]
  example_urls: [https://www.sudokuwiki.org/Remote_Pairs]
  slop_notes: "what add-chute-remote-pairs ACTUALLY built (its own UI says 'Remote Pairs'). Correct for what it is, but sudokuwiki lists plain Remote Pairs as DEPRECATED. Rename/rescope honestly or discard"

- strategy: XCycles
  family: Chaining
  grade: Diabolical
  status: slop-defective
  requires_primitives: [strong_link, weak_link, candidate_link_graph, sees_predicate]
  builds_on: [SimpleColouring]   # spine: "X-Cycles are strongly related to Simple Coloring"
  example_urls: [https://www.sudokuwiki.org/X_Cycles]              # Part 2 (discontinuous) UNKNOWN (not fetched)
  slop_notes: "continuous-loop rule matches spine; discontinuous handling UNCERTAIN. Concrete defect: dedups cycles by sorted node-SET not edge-sequence, silently dropping distinct cycles. O(len^2) path clones. OFF by default"

- strategy: XYChains
  family: Chaining
  grade: UNKNOWN
  status: absent
  requires_primitives: [bivalue_cell, candidate_link_graph, sees_predicate]
  builds_on: [SimpleColouring]
  example_urls: []
  slop_notes: "unattempted; needs the (Position,Value)-node link graph"

- strategy: BruteForce
  family: Backend
  grade: n/a
  status: implemented
  requires_primitives: [uniqueness_oracle]
  builds_on: []
  example_urls: []
  slop_notes: "delegates to introspective SAT/backtracking solver; not a human strategy"
```

---

## 2. Shared-infrastructure proposals

Full version with evidence citations: `.tmp/3a-hierarchy-infra.md` Part (b). Ordered by leverage. Shapes only — implementations are yours.

### P1 `GroupCandidateAvailability` — persistent transposed group view
- **Evidence:** duplicated twice on `dev` (`x_wing.rs:221-259`, `group_intersection/mod.rs:127-175`), reinvented byte-for-byte a third time by the Swordfish branch. The `add-persistent-group-availability` branch already formalized the struct and refactored XWing + GroupIntersection onto it, behavior-preserving.
- **Home/shape:** `solver/strategic/group_candidate_availability.rs`; `StrategicGroupAvailability<Base>` (one `GroupCandidateAvailability` per Value: `rows`/`columns`/`row_major_blocks`/`column_major_blocks`, each `CandidatesGroup<Base>`), threaded via additive `Strategy::execute_with_availability(grid, &avail)` defaulting to `execute`.
- **Leverage:** ~8 strategies (HiddenSingles, LockedSets, GroupIntersection, XWing, Swordfish, Jellyfish, BUG's group counting) **plus** the link-graph tier derives strong links from it (`count()==2` popcount).
- **Open questions (yours):** incremental-vs-rebuild — the branch's incremental API is dead code (zero call sites); its benchmarks time `from_grid` in isolation and answer nothing. See roadmap Step 3. Also: block index→Position still goes through the separate `BlockSegment` API — a seam any block-link consumer must bridge.

### P2 `Position::sees` predicate
- **Evidence:** 7 named + 1 inlined reimplementations across branches; y_wing/xyz_wing versions return `true` for `pos == pos` (bug masked by call-site guards).
- **Home/shape:** `position/mod.rs`: `fn sees(self, other: Self) -> bool`, self-guard baked in (`pos.sees(pos) == false`). `sees_both`/`sees_any_of` are trivial folds, no own home.
- **Leverage:** ~10 strategies (all Bent Sets, all chains/colouring, UR). Zero design risk.

### P3 Bivalue / candidate-count cell index
- **Evidence:** ad-hoc count scans in ≥4 branches; `dev` already has `Grid::all_candidates_positions()` — the count filter is the missing piece. This is the per-cell view P1 explicitly does not serve.
- **Home/shape:** `grid/mod.rs`: `fn positions_with_candidate_count(&self, count: u8) -> Vec<(Position<Base>, Candidates<Base>)>` + `bivalue_positions()` wrapper. Count-parameterized because XYZ-Wing needs trivalue and BUG needs "exactly one trivalue".
- **Leverage:** ~8 (wings, UR, BUG, Remote Pairs; unlocks Chute Remote Pairs, XY-Chains).

### P4 Candidate link graph (strong/weak links)
- **Evidence:** four incompatible builds (w-wing flat pair list, simple-colouring 2-colouring adjacency, x-cycles typed graph + alternating DFS, remote-pairs bipartite colouring); all four hand-roll the conjugate scan P1 answers.
- **Home/shape:** new module under `solver/strategic/`, layered on P1: `CandidateLinkGraph<Base>` with `strong_links(candidate)` flat iterator (serves W-Wing without a walk) and `neighbors(candidate, pos, kind)`. Colouring/cycle algorithms stay outside as strategy-local functions — the struct is a passive index.
- **Leverage:** gateway to the whole Diabolical/Chaining tier (Simple Colouring, X-Cycles, Rectangle Elimination, XY-Chains, 3D Medusa).
- **Open questions (yours):** `Position` nodes (v1, covers all current branches) vs `(Position, Value)` nodes from the start (needed by XY-Chains/3D Medusa; subsumes v1 as a slice). Dedup contract for pairs linked via two unit types. Inherits P1's persistence question.

### P5 Generic `fish(n)` scan
- **Evidence:** X-Wing (n=2) and Swordfish (n=3) are the same algorithm; the spine frames X-Wing→Swordfish→Jellyfish as one pattern.
- **Home/shape:** free function over P1: `fish_scan(avail, candidate, axis, n) -> Deductions` — filter lines to `2..=n` positions, enumerate `C(lines, n)`, keep unions of size exactly n, eliminate on covering lines outside base lines.
- **Leverage:** 3 strategies in one implementation; Jellyfish free.
- **Open question (yours):** migrate the tested X-Wing onto n=2 or leave it bespoke.

### P6 Uniqueness-oracle plumbing (SAT) — placed, design in §3
Consumed by UR/BUG pre-checks and future Uniqueness-family strategies. Deferred: orthogonal to the strategy-graph work.

---

## 3. SAT-fit assessment

Full version incl. cost model: `.tmp/3b-sat-fit-roadmap.md` Deliverable (c).

**The core distinction:** SAT is a boolean oracle over the whole grid. It fits when your question reduces to "does a complete valid assignment exist (or exist ≥2 ways)?" and a yes/no suffices. It does not fit when the human-legible pattern is the deliverable — there SAT *replaces* the strategy instead of pre-checking it, destroys the `Reason` payload, and removes the difficulty-grading signal.

| Item | Verdict | Reason |
|---|---|---|
| uniqueness_oracle (P6) | **partial-logic enabler** | The one reusable piece; all good uses route through it. |
| Unique Rectangles | **pre-check** | Gate deadly-pattern eliminations on a real uniqueness check instead of the unchecked axiom; the human still finds the rectangle. |
| BUG | **pre-check** | Same shape: BUG+1 is meaningless without uniqueness. |
| BruteForce | good fit, done | Whole-grid solve is the canonical SAT question (Base4/5 dispatch exists). |
| Generator grading (`SatStepCount`) | **borderline** | Cost-as-signal, not truth value; keep distinct from the sudokuwiki grade. |
| Fish / wings / chains / colouring | **not-applicable** | Pattern search is the learning content. SAT's `is_forced` would find a superset of every chain elimination — reject it anyway: it collapses strategy distinctions and yields no Reason. |
| P1–P5 primitives | **not-applicable** | Geometric/bitset indices; no satisfiability question. |

**Key capability gap (verified):** `sat::Solver` fixes its assumptions at construction — `assume()` is private and called once; changing the denied-candidate set means a fresh CNF clone, discarding learned clauses. The incremental assume+re-solve pattern exists only bespoke inside the pruning branch's `AmbiguousSolutionChecker`, which additionally relies on a *known target solution* (single blocking clause) — a shortcut unavailable at strategy time, where "≥2 solutions" needs `SolverIter` model-negation (≥2 solves). No unsat-core read exists anywhere.

**Missing public API (shapes only):**

```rust
// Layer 1 — make the live solver re-queryable
impl<Base: SudokuBase> Solver<Base> {
    fn reassume<Filter: CandidatesFilter<Base>>(&mut self, grid: &Grid<Base>, filter: &Filter);
    fn is_satisfiable(&mut self) -> Result<bool>;
}

// Layer 2 — strategy-facing oracle (P6 home: solver/sat/); no known-solution blocking clause
struct SolutionOracle<Base: SudokuBase> { /* one cloned base CNF + fixed givens */ }
impl<Base: SudokuBase> SolutionOracle<Base> {
    fn new(grid: &Grid<Base>) -> Self;
    fn admits(&mut self, pos: Position<Base>, value: Value<Base>) -> Result<bool>;
    fn is_forced(&mut self, pos: Position<Base>, value: Value<Base>) -> Result<bool>;
    fn has_multiple_solutions(&mut self) -> Result<bool>;   // budget once-per-grid, never per-candidate
}
// flagged only: failing_assumptions() for SAT-derived Reason payloads — unplumbed, varisat support unverified
```

**Cost model:** N fresh solvers = N CNF clones + N assumption rebuilds + zero retained learning; incremental = 1 clone + N cheap assumption swaps with varisat's learned clauses persisting. That delta is exactly what the pruning branch exploits — but its efficiency claims are prose-only; build P6 with a benchmark, not inherited claims.

**Heuristics to generalize from:** (1) question reduces to whole-grid satisfiability? → SAT; (2) pattern is the deliverable? → not SAT, even if it finds the same eliminations; (3) boolean answer vs cost-as-signal → oracle vs borderline; (4) many small variations on one grid → incremental assume+solve (doesn't publicly exist yet); (5) known target solution (generator) vs none (strategy time) — the generator's cheap trick does not transfer.

---

## 4. Roadmap — next steps

Full scoring rationale: `.tmp/3b-sat-fit-roadmap.md` Deliverable (d). Criteria: (1) infra leverage, (2) sudokuwiki progression, (3) prerequisite readiness, (4) reference-test-case availability, (5) generator/grading payoff. All-candidate scores (1–5, unweighted Σ for visibility):

| Candidate | (1) | (2) | (3) | (4) | (5) | Σ |
|---|:-:|:-:|:-:|:-:|:-:|:-:|
| Y-Wing | 2 | 5 | 5 | 5 | 3 | **20** |
| P2 `Position::sees` | 5 | 3 | 5 | 5 | 1 | **19** |
| P1 GroupCandidateAvailability | 5 | 3 | 4 | 4 | 3 | **19** |
| P3 bivalue/count index | 4 | 3 | 5 | 5 | 1 | **18** |
| P5 `fish(n)` scan | 4 | 4 | 3 | 3 | 4 | **18** |
| Swordfish | 2 | 4 | 3 | 2 | 4 | 15 |
| XYZ-Wing | 1 | 4 | 4 | 4 | 2 | 15 |
| BUG | 1 | 3 | 3 | 4 | 4 | 15 |
| P4 candidate_link_graph | 5 | 2 | 2 | 2 | 2 | 13 |
| Chute Remote Pairs | 1 | 3 | 3 | 3 | 2 | 12 |
| Unique Rectangles | 1 | 3 | 3 | 2 | 3 | 12 |
| Rectangle Elimination | 1 | 3 | 2 | 3 | 3 | 12 |
| W-Wing | 1 | 3 | 2 | 3 | 2 | 11 |
| Simple Colouring | 2 | 3 | 2 | 1 | 3 | 11 |
| Jellyfish | 1 | 3 | 2 | 1 | 3 | 10 |
| P6 uniqueness_oracle | 2 | 2 | 1 | 3 | 2 | 10 |
| X-Cycles | 1 | 2 | 1 | 2 | 3 | 9 |
| Remote Pairs | 1 | 2 | 2 | 1 | 1 | 7 |
| XY-Chains | 1 | 2 | 1 | 1 | 2 | 7 |

### Step 1 — `Position::sees` (P2) + candidate-count index (P3)
Two small per-cell primitives (shapes in §2). Reference: the 8 duplicate sees-helpers; **avoid** the y_wing/xyz_wing self-comparison bug — bake `pos.sees(pos) == false` in. Test: pure unit tests (truth table incl. self-equality; hand-built grid for the count index). Unlocks Y-Wing and every later Bent-Set/UR/chain consumer. Beats starting with P1: equal leverage, zero design risk, and it clears the most-blocked path first.

### Step 2 — Y-Wing, on the new primitives
Bivalue cells via P3; for each pivot `{X,Y}` find house-sharing bivalue pincers with leftover `Z1 == Z2 == Z`; eliminate Z from cells seeing both pincers. No persistent index needed — per-`execute()` scan suffices. Reference: `copilot/implement-y-wing-strategy` (spine-correct logic) but **write the full test template the branch omitted** — it has no `assert_deductions`, no snapshot macro, no reference puzzle. Test: https://www.sudokuwiki.org/Y_Wing_Strategy — concrete coordinates captured: pivot E1 `{7,2}`, pincers A1/A7, eliminate 2 from E7 (rare — most Band-2 spine examples were not captured). Cheap follow-on: XYZ-Wing (also concrete: hinge F9, eliminate 1 from F7). Beats Chute Remote Pairs (spine's teaching order puts it first) because that branch built the wrong strategy — no correct reference exists — and it's a niche technique.

### Step 3 — Persistent `GroupCandidateAvailability` (P1)
This resolves the existing Notion task. Reference: `copilot/add-persistent-group-availability` — the struct extraction and XWing/GroupIntersection refactor are faithful and behavior-preserving (keep), but:
- **Start rebuild-once-per-solver-step, not incremental.** The branch's incremental API is dead code (zero call sites; never wired into `Action::apply`). Only wire incremental + hold as `SolverPathIter` state if the benchmark justifies it.
- **Benchmark shape:** (a) criterion bench of a full `SolverPathIter` solve, shared-availability vs today's per-strategy rebuild; (b) only if `from_grid` dominates, incremental-vs-rebuild on a mid-solve grid. Decision gate: no incremental wiring until (a) demands it.
- Watch the block-position seam (`BlockSegment` API).
Test: behavior-preserving — existing X-Wing/GroupIntersection sudokuwiki tests + snapshots stay green unchanged. Unlocks Step 4 and the future link graph (strong link = `count()==2`).

### Step 4 — Generic `fish(n)` (P5) → Swordfish (+ Jellyfish free)
One scan over P1 (shape in §2); Swordfish = n=3, Jellyfish = n=4 free; X-Wing migration to n=2 is your call (touches a tested strategy). Reference: `copilot/add-swordfish-strategy` for the n=3 logic (sound); **avoid** perpetuating its verbatim-copied index (build on P1) and its loose synthetic tests (a wrong axis boundary would pass) — write exact `assert_deductions` cases. Test: https://www.sudokuwiki.org/Sword_Fish_Strategy (fetch concrete example coords when implementing — spine didn't capture them); interim, verify X-Wing's existing reference eliminations reproduce through the generic scan. Beats jumping to the link graph: far higher readiness, self-contained, direct grading payoff.

### Step 5 — Housekeeping: mine, then delete
Hand-port `add-locked-sets-reasoning`'s Reason::Candidates fix (fills the existing `// TODO: produce decuction reasons`; ~95% of its diff is snapshot churn). Confirm nothing else is needed from the XYZ-Wing branch. Then delete all mined `copilot/*` solver branches so the duplication can't resurface.

### Deferred (below the cut, in order)
- **P4 link graph → W-Wing, Simple Colouring, X-Cycles, Rectangle Elimination, XY-Chains:** highest structural leverage, lowest readiness (node-type decision up front; consumers have uncaptured spine examples and defective slop). Natural Step 6, layered on P1.
- **P6 uniqueness oracle → UR/BUG pre-checks:** needs the Layer-1/2 SAT API (§3) built and benchmarked first; lands when the Uniqueness family is the focus.

---

## Appendix — per-branch gotchas (crosscheck before deleting branches)

Verify any claim via `git show origin/copilot/<branch>:<path>`; full reports in `.tmp/2-*.md`.

| Branch | Verdict | Key gotcha |
|---|---|---|
| implement-y-wing | correct | no self-guard in `shares_house`; weakest tests of all wings |
| implement-xyz-wing | correct | un-enforced "same chute" question (spine ambiguous); no reference-puzzle assert |
| add-w-wing | correct (base case only) | missing spine variants; duplicate-pair emission in StrongLinksMap |
| add-swordfish | correct | verbatim x_wing.rs index copy; loose synthetic tests would miss axis bugs |
| add-simple-colouring | correct (Rules 2,4) | Rule 7 omitted; no reference puzzle, no snapshot macro |
| implement-x-cycles | defective | cycle dedup by node-set drops distinct cycles; discontinuous rule unverified; off by default |
| implement-unique-rectangles | defective | Type 4 floor-side elimination likely unsound + untested; dead Type-2 test assertion |
| add-rectangle-elimination | mislabeled | actually UR Types 1/2 (good skeleton); real Rectangle Elimination never built |
| implement-new-bug | correct | no ambiguity guard for >1 count==3 candidate; thin tests |
| add-chute-remote-pairs | mislabeled | built deprecated generic Remote Pairs; zero chute logic; over-engineered graph machinery |
| add-locked-sets-reasoning | correct | worth hand-porting; ~95% snapshot churn |
| add-persistent-group-availability | half-delivered | incremental API is dead code; benchmarks don't answer the bottleneck question |
| optimize-sat-pruning | useful evidence | incremental SAT pattern is bespoke + relies on known-solution shortcut; efficiency claims unmeasured |

### Research artifact index (`.tmp/`, review artifacts — not cleaned up)
- `0-manifest.md` — branch refs, diff base, file assignments
- `0-branch-survey.md`, `0-codebase-baseline.md` — session exploration
- `1-theory-spine.md` — sudokuwiki theory, cited per claim, `UNKNOWN` = page didn't state it
- `2-wings|fish|chains|unique-rectangles|misc|infra-group-availability|sat-pruning.md` — per-branch extraction
- `3a-hierarchy-infra.md` — hierarchy + primitives (human-reviewed)
- `3b-sat-fit-roadmap.md` — SAT assessment + scored roadmap
