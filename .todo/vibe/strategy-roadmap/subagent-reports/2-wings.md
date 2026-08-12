# Phase 2 — WINGS group (Y-Wing, XYZ-Wing, W-Wing)

Diff base for all three: `b87a1b3a` (merge-base with `origin/master`). Read via `git diff b87a1b3a...origin/copilot/<branch>` and `git show origin/copilot/<branch>:<path>` only; no checkout.

---

## origin/copilot/implement-y-wing-strategy

**File**: `sudoku-rs/src/solver/strategic/strategies/impls/y_wing.rs` (new, 405 lines). Registers `YWing` in `strategy_enum.rs`, score `160`.

### 1. Correctness check vs Theory Spine
**Verdict: yes** (matches the spine's Y-Wing entry). Pivot = bivalue cell `{X,Y}`; wings collected from bivalue cells sharing a house with the pivot and sharing exactly one candidate with it (`shared_candidates.count() != 1` is rejected, which also transitively guarantees the wing's "off" candidate Z is never X or Y). Wing1 keyed off shared==X, Wing2 off shared==Y; both wings' remaining candidate must agree (`z1 == z2`) before elimination. Eliminates Z from any cell distinct from all three pattern cells that shares a house with *both* wings. This is exactly the spine's AB/AC/BC pattern — pivot sees both pincers, pincers need not see each other, eliminate the shared non-pivot candidate from cells seeing both pincers. No precondition/elimination mismatch found.

Score `160` is an unexplained ad hoc placeholder (comment claims parity with X-Wing's 200 but doesn't derive it from the spine's Grading Puzzles numbers, which don't list a Y-Wing point value anyway) — consistent with the rest of the ad hoc scoring already on `dev` (baseline `strategies/mod.rs:23` TODO), not a new defect.

**Reference URLs in test code: none.** No `sudokuwiki.org` URL appears anywhere in the file (only a bare parenthetical "According to sudokuwiki.org" in a scoring comment, no link, no example puzzle). The spine's canonical Y-Wing example (pivot E1 `{7,2}`, pincers A1 `{7,1}` / A7 `{1,2}`, eliminate 2 from E7) is **not used**.

**Test quality vs dev template**: weak. No `assert_deductions`/`assert_deductions_with_grid` calls at all — every test hand-builds a small grid and asserts via `deductions.iter().any(|d| *d == expected)` or manual predicate closures instead of the standard helper. No `strategy_snapshot_tests!` macro invocation — this strategy has **no per-strategy insta snapshot coverage** (the only snapshot diffs in this branch are solver-level `solve_path_all` snapshots that changed because YWing is now in the default strategy set, not dedicated YWing snapshots). Weakest test-infra compliance of the three WINGS branches: misses template steps 2 and 3 entirely, only has step 1 (synthetic unit tests), and even those are ad hoc rather than the `Deduction`-object-equality idiom used elsewhere.

### 2. Codebase-gap findings
- No new persistent data structures; operates on a `Vec<(Position<Base>, Candidates<Base>)>` collected fresh from `Position::<Base>::all()` each call (full-grid scan every `execute()`, same pattern as the existing `GroupCandidateIndexes` duplication already flagged on `dev`).
- Local helper `fn shares_house<Base: SudokuBase>(pos1: Position<Base>, pos2: Position<Base>) -> bool` — see cross-branch section.
- No new test infra, no SAT usage.

### 3. Pattern description for the owner
Collect every bivalue cell (exactly 2 remaining candidates) on the grid. For each candidate pivot cell `{X,Y}`, scan the other bivalue cells for ones sharing a row/column/block with the pivot and sharing exactly one of `{X,Y}` — these are candidate wings. Split them into "shares X" wings (remaining candidate is Z1) and "shares Y" wings (remaining candidate is Z2). Any (wing-sharing-X, wing-sharing-Y) pair whose leftover candidates agree (Z1 == Z2 == Z) forms a valid Y-Wing; eliminate Z from every other cell that shares a house with *both* wings and still has Z as a candidate. Needs only per-cell candidate counts and house-membership (row/column/block) — no persistent index required, a fresh full-cell scan per call is sufficient (that's what this branch does). Gotcha to avoid repeating: the house-sharing predicate here returns `true` for identical positions (no self-check baked in), so every call site must independently exclude `pos == other` — fragile if the helper is reused without that discipline (see cross-branch section for the fix). Also: don't skip validating against the two spine example puzzles (Y-Wing's own worked example, and ideally an XY-Chain-adjacent negative case) — this branch's tests are all synthetic hand-built minimal grids, giving no confidence the implementation matches sudokuwiki's own worked numbers.

---

## origin/copilot/implement-xyz-wing-strategy

**File**: `sudoku-rs/src/solver/strategic/strategies/impls/xyz_wing.rs` (new, 364 lines). Registers `XyzWing` in `strategy_enum.rs`, score `300`.

### 1. Correctness check vs Theory Spine
**Verdict: yes**, with one **uncertain** nuance. Hinge = tri-value cell `{X,Y,Z}`; wings = bivalue cells that are subsets of the hinge's candidates and share a house with the hinge (`sees_each_other`). For each wing pair: requires distinct candidate sets, requires their union to equal the full hinge candidate set (i.e. together `{X,Z}∪{Y,Z} = {X,Y,Z}`), and requires the three-way intersection to reduce to exactly one candidate Z (`to_single()`). Eliminates Z from any cell (excluding the three pattern cells) that sees *all three* of hinge/wing1/wing2 and still carries Z. This matches the spine's XYZ-Wing description and elimination rule exactly.

The spine's prose also says "the three cells fall within the same 'chute' (row/column plus box configuration)" as part of its precondition description; this branch does **not** independently enforce a chute constraint — it only requires pairwise hinge-to-wing visibility (`sees_each_other`, not requiring wing1-sees-wing2). Flagging as `uncertain` rather than a defect: it's unclear from the fetched spine excerpt whether "same chute" is a true independent precondition or just an incidental description of the worked example's geometry (the elimination-cell logic — "sees all three" — is standard and doesn't itself require a chute). Worth the owner double-checking against a second source before treating either branch's interpretation as ground truth.

Score `300` is again an unexplained ad hoc placeholder, same caveat as Y-Wing.

**Reference URLs**: module doc-comment cites `https://www.sudokuwiki.org/XYZ_Wing` (page-level, not example-level). The spine's specific worked example (hinge F9, pincers D9 `{1,2}` / F1 `{1,4}`, eliminate 1 from F7) is **not used** in tests — all three unit tests use their own synthetic minimal grids with hand-picked coordinates.

**Test quality vs dev template**: partial. Uses `strategy_snapshot_tests!(XyzWing)` (template step 3 — present, unlike Y-Wing and W-Wing). But step 2 is not followed in the strict sense: no `assert_deductions`/`assert_deductions_with_grid` against a literal sudokuwiki example board; instead, three synthetic unit tests assert individual eliminations via `deductions.iter().any(...)` predicate closures. Better than Y-Wing (has snapshots) and W-Wing (has neither snapshots nor sudokuwiki examples), but still short of the full template.

### 2. Codebase-gap findings
- No new persistent data structures. Reuses the **existing** `Grid::all_candidates_positions()` (already on `dev`, `grid/mod.rs:729`) rather than reinventing a full-grid position scan — a positive example of infra reuse, unlike Y-Wing's inline `Position::all()` filter.
- Local helper `fn sees_each_other<Base: SudokuBase>(a: Position<Base>, b: Position<Base>) -> bool` — see cross-branch section (same semantics/defect shape as Y-Wing's `shares_house`).
- Inline bi/tri-value cell categorization: single loop over `all_candidates_positions()` bucketing into `bi_value_cells: Vec<(Position, Candidates)>` (count==2) and `tri_value_cells: Vec<(Position, Candidates)>` (count==3) — no shared helper, but a natural generalization target (see cross-branch section).
- No SAT usage.

### 3. Pattern description for the owner
Scan all cells once, bucketing into "exactly 2 candidates" (potential wings) and "exactly 3 candidates" (potential hinges). For each hinge cell `{X,Y,Z}`, collect all bivalue cells that are (a) a candidate-subset of the hinge and (b) share a house with the hinge. For every unordered pair of such wings with different candidate sets whose union equals the hinge's full candidate set, compute the three-way candidate intersection; if it collapses to a single value Z, that's a valid XYZ-Wing. Eliminate Z from every other cell that shares a house with the hinge AND both wings simultaneously (three-way visibility, not just pairwise). Needs per-cell candidate sets and house-sharing only — no persistent index needed, `O(hinges × wings²)` per call. Gotcha: this branch's `sees_each_other` (like Y-Wing's `shares_house`) has no built-in self-equality guard, so every filter that calls it must separately exclude identical positions — it does so correctly today (`wing_pos != pivot_pos`, `pos != pivot_pos && pos != wing1_pos && pos != wing2_pos`), but that's an easy invariant to drop if this code is refactored. Also worth the owner's attention: verify independently whether a "same chute" constraint is theoretically required beyond three-way visibility (spine wording is ambiguous on this point, see above) before assuming this implementation's scope is complete.

---

## origin/copilot/add-w-wing-strategy

**File**: `sudoku-rs/src/solver/strategic/strategies/impls/w_wing.rs` (new, 406 lines). Registers `WWing` in `strategy_enum.rs`, score `250`.

### 1. Correctness check vs Theory Spine
**Verdict: yes**, for the "Single W-Wing" variant only. Two bivalue cells with an identical candidate pair `{x,y}` that do **not** see each other (`cells_see_each_other` check, negated) are candidate endpoints. For each of the two candidates as the putative "link candidate," the code searches all precomputed strong links (grid positions with exactly 2 occurrences of a candidate within some row/column/block) for one whose two endpoints respectively see cell A and cell B (checked both orderings). If found, the *other* candidate is eliminated from every cell seeing both A and B. This matches the spine's core W-Wing precondition/elimination rule precisely (bivalue pair, no direct sight, strong-link bridge on one shared candidate, eliminate the other shared candidate from common peers).

**Scope gap** (not a bug, but incomplete per spine): the spine names three named variants — Single W-Wing (implemented), Double/Remote Pair Chain (a longer alternating chain of remote bivalue pairs), and Split Double (two simultaneous W-Wings with shared endpoints). This branch implements **only** the two-cell/single-strong-link case; the chain variants are out of scope. The owner should treat this as "W-Wing base case only," not full W-Wing coverage.

Score `250` is again an unexplained ad hoc placeholder (comment says "slightly more difficult than X-Wing (200)" without deriving from spine data — spine gives no W-Wing point value).

**Reference URLs**: module doc-comment cites `https://www.sudokuwiki.org/W_Wing_Strategy` (page-level). The spine's specific worked example (A6/F5 both `{3,6}`, strong link on 6, eliminate 3 from D6/E6) is **not used**; both tests are synthetic hand-built grids.

**Test quality vs dev template**: weakest alongside Y-Wing. No `assert_deductions`/`assert_deductions_with_grid` usage — assertions are manual `deductions.iter().any(...)` predicate checks. **No `strategy_snapshot_tests!` macro invocation at all** (grepped for it in the file — zero matches); the branch's only snapshot diffs are solver-level `solve_path_all` re-recordings from WWing joining the default strategy set, not a dedicated WWing snapshot suite. Also has an unused/dead helper: `w_wing_deduction` test-builder function is marked `#[allow(dead_code)]` and never called by any test — a sign the test suite doesn't actually build full expected-`Deduction` objects for comparison anywhere.

### 2. Codebase-gap findings
- New data structures: `struct StrongLink<Base> { pos1: Position<Base>, pos2: Position<Base> }` (unordered position pair) and `type StrongLinksMap<Base> = Vec<Vec<StrongLink<Base>>>` (outer index = candidate value − 1, inner = every strong link found for that candidate across every row/column/block in the grid, rebuilt fresh every `execute()` call).
- Helper: `fn compute_strong_links<Base: SudokuBase>(grid: &Grid<Base>) -> StrongLinksMap<Base>` — one pass over `Grid::all_group_positions()` (existing dev helper) × every candidate value, pushing a `StrongLink` whenever a group has exactly 2 positions holding that candidate.
- Helper: `fn try_find_w_wing_with_link<Base>(grid, pos_a, pos_b, link_candidate, eliminate_candidate, strong_links: &StrongLinksMap<Base>) -> Option<Deduction<Base>>`.
- Local sees-helper `fn cells_see_each_other<Base: SudokuBase>(pos1, pos2) -> bool` — see cross-branch section; this one *does* self-check (`pos1 == pos2 → false`), unlike the other two branches' helpers.
- Uses the `itertools` crate's `.tuple_combinations()` on the bivalue-cell list (not used elsewhere in the three WINGS branches) — worth noting as a dependency-usage precedent if the owner wants unordered-pair iteration idioms.
- No SAT usage.

### 3. Pattern description for the owner
First, precompute every "strong link" in the grid: for each candidate value and each row/column/block, if that unit has exactly two remaining positions for the candidate, record the pair. Then collect all bivalue cells and look at every unordered pair `(A, B)` sharing an identical candidate pair `{x,y}` that do not see each other. For each of the two candidates in turn (call it the link candidate, the other is the elimination candidate), scan the precomputed strong links for that link candidate for one whose two endpoints are distinct from A and B and respectively see A and see B (either assignment order). If such a link exists, every cell that sees both A and B (excluding the four pattern cells) loses the elimination candidate. Needs per-cell candidate counts, house-sharing, and one full-grid strong-link precomputation before the pairwise scan can start — the strong-link table is grid-global, not local to A/B, so it should be computed once per `execute()` call, not once per candidate pair. Gotchas: (1) this branch's strong-link builder can push the same physical position-pair twice if it happens to be simultaneously the sole occurrence in two different unit types (e.g. same row and same block) — harmless here since the consumer only checks non-emptiness, but a footgun for any future consumer that assumes deduplication or counts links; (2) only the two-cell "Single W-Wing" case is covered, not the remote-pair-chain or split-double variants named in the spine; (3) no sudokuwiki worked example is used in tests, so numeric correctness against the canonical example (A6/F5 `{3,6}` → eliminate 3 from D6/E6) is unverified.

---

## Cross-branch: sees/bivalue primitives (owned finding)

### "Sees each other" predicate
All three branches reimplement the same idea under different names, with one real semantic difference:

| Branch | Name | Self-cell (`pos == pos`) | Row/col/block check |
|---|---|---|---|
| y_wing | `fn shares_house<Base>(pos1, pos2) -> bool` | **returns `true`** (no guard — row, col, block all trivially equal) | plain OR of the three equalities |
| xyz_wing | `fn sees_each_other<Base>(a, b) -> bool` | **returns `true`** (same, no guard) | identical OR |
| w_wing | `fn cells_see_each_other<Base>(pos1, pos2) -> bool` | **returns `false`** (explicit early-return guard) | identical OR, guarded |

No precedence differences exist otherwise (all three are a pure `row == row || column == column || block == block`, so block vs row/col ordering is irrelevant — OR is commutative here). The only real edge-case divergence is same-cell handling. In y_wing and xyz_wing this is masked because every call site independently adds `pos != other` guards before calling the helper — but that's fragile: any future call site (or a refactor into a shared helper) that forgets the external guard silently starts treating a cell as "seeing itself," which would let a strategy eliminate a cell's own candidate as if some peer forced it, or count a cell as its own pattern participant.

**Recommendation**: one canonical primitive, self-check baked in (matching w_wing's safer behavior), as an inherent method on `Position` rather than a free function per strategy module:

```
impl<Base: SudokuBase> Position<Base> {
    pub fn sees(self, other: Self) -> bool
}
```
Home: `sudoku-rs/src/position/mod.rs`, next to the existing `to_row`/`to_column`/`to_block` accessors it composes. This lets every consumer (wings, and per the branch-survey report, x_cycles' `sees`/`sees_both`, chute_remote_pairs' `positions_see_each_other`, simple_colouring's `sees_any_of`, unique_rectangle's `shares_group`) drop their local copies. Signature-only note for the owner: `sees_any_of`/`sees_both` in the other agents' branches are trivial folds over this single primitive and don't need their own home.

### Bivalue-cell detection
y_wing and xyz_wing each inline the same scan (`Position::all()` or `all_candidates_positions()` filtered by `candidates().count() == N`) — y_wing for N=2, xyz_wing for N=2 *and* N=3 (bucketed together in one pass). The natural canonical shape, generalizing both call sites, is a `Grid`-level accessor parameterized on the candidate count rather than a bivalue-specific one (xyz_wing already needs trivalue too, and other branches in the survey — chute_remote_pairs — need bivalue detection again):

```
impl<Base: SudokuBase> Grid<Base> {
    pub fn positions_with_candidate_count(&self, count: u8) -> Vec<(Position<Base>, Candidates<Base>)>
}
```
Home: `sudoku-rs/src/grid/mod.rs`, alongside the existing `all_candidates_positions()` it would be built from (filter + map). A `bivalue_positions()` convenience wrapper (`count == 2`) can sit on top for readability at call sites, but the count-parameterized form is the one that avoids yet another near-duplicate for the next strategy that needs trivalue-or-N-value scanning.

### W-Wing's `StrongLink`/`StrongLinksMap` — what W-Wing minimally needs
Shape: `StrongLink<Base> { pos1: Position<Base>, pos2: Position<Base> }` (an unordered position pair, no unit-type tag, no strong-vs-weak distinction — every entry is definitionally strong by construction) plus `StrongLinksMap<Base> = Vec<Vec<StrongLink<Base>>>` indexed by candidate value, built by one full-grid pass over `Grid::all_group_positions()` counting per-unit occurrences.

This is a much narrower need than what the chains-family branches build for the same underlying concept (per the branch-survey report: x_cycles' `CandidateGraph`/`Edge`/`LinkType`/`AlternatingPath` with strong *and* weak links plus cycle-finding, and simple_colouring's `Color`/`build_color_chains` two-coloring traversal). W-Wing never needs: link direction, weak links, a graph/adjacency-list traversal, or cycle detection — it only ever asks "for candidate v, what are all the strong-linked position pairs anywhere on the board?" and then does a flat O(links) scan per bivalue-pair candidate.

**Recommendation for the chains agent's reconciliation**: whatever shared link/graph structure gets built, W-Wing should be able to consume just its strong-link enumeration surface, e.g. an accessor shaped like `.strong_links_for(candidate: Value<Base>) -> impl Iterator<Item = (Position<Base>, Position<Base>)>` — it should not need to construct or traverse the full graph/cycle machinery to get this. Also flag to that agent: this branch's `compute_strong_links` can emit the same physical pair twice (once per unit type it's simultaneously the sole occupant of, e.g. a same-row-and-same-block pair) since it doesn't dedupe across row/column/block passes — the canonical structure should either dedupe (store as a set) or the API should make clear to consumers that duplicates are possible and harmless-if-only-checking-non-emptiness but wrong if a consumer ever counts or enumerates-uniquely.
