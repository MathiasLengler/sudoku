# Phase 0 — Copilot Branch Survey (diffstat-level)

Produced by a Sonnet Explore agent during planning; verified against `git diff --stat`/`git log`. Diff base for all branches: `b87a1b3a` (identical merge-base with `origin/master` — every agent forked from the same point on `dev`; true parallel siblings).

## Per-branch reports

### origin/copilot/add-locked-sets-reasoning
- 21 files, +4704/-782 (mostly snapshot re-recording).
- Modifies existing `strategies/impls/locked_sets.rs` and `locked_sets/v2.rs` — NOT a new strategy; adds deduction-*reason* detail to Locked Sets (intersects locked candidates with actual cell candidates, updates snapshots). Does not touch `strategy_enum.rs`.
- No new shared infra.

### origin/copilot/implement-y-wing-strategy
- 9 files, +2004/-2. Adds `strategies/impls/y_wing.rs`, modifies `impls/mod.rs`, `strategy_enum.rs`. Tests + snapshots.
- Y-Wing (XY-Wing). Filters bi-value cells via `candidates.count() == 2`; local helper `fn shares_house<Base>(pos1, pos2) -> bool`.

### origin/copilot/implement-xyz-wing-strategy
- 22 files, +2160/-4. Adds `impls/xyz_wing.rs` + full snapshot set; modifies `impls/mod.rs`, `strategy_enum.rs`. Tests.
- XYZ-Wing. Local helper `fn sees_each_other<Base>(a, b) -> bool` (same concept as Y-Wing's `shares_house`, different name).

### origin/copilot/add-w-wing-strategy
- 8 files, +571/-4. Adds `impls/w_wing.rs`; modifies `impls/mod.rs`, `strategy_enum.rs`. Tests.
- W-Wing. Own strong-link infrastructure: `struct StrongLink<Base>`, `StrongLinksMap<Base>`, `fn compute_strong_links`, plus sees-helper `fn cells_see_each_other`.

### origin/copilot/add-swordfish-strategy
- 11 files, +1776/-4. Adds `impls/swordfish.rs`; modifies `impls/mod.rs`, `strategy_enum.rs`. Tests.
- Swordfish. Adds private `struct GroupCandidateIndexes<Base>{ rows: CandidatesGroup<Base>, columns: CandidatesGroup<Base> }` — ad hoc transposed candidate availability per row/column — plus `SwordfishPattern`, `Axis` enum, `find_swordfish_axis`.

### origin/copilot/add-simple-colouring-strategy
- 11 files, +3918/-3. Adds `impls/simple_colouring.rs`; modifies `impls/mod.rs`, `strategy_enum.rs`. Tests.
- Simple Colouring. Own chain/graph infra: `enum Color`, `fn build_color_chains`, `fn build_chain_from_position`, `fn find_all_strong_links`, `fn find_candidate_positions`, `fn has_color_conflict`, `fn sees_any_of`, Type1/Type2 elimination.

### origin/copilot/implement-x-cycles-strategy
- 14 files, +3094/-1. Adds `impls/x_cycles.rs` + full snapshot set; modifies `impls/mod.rs`, `strategy_enum.rs`. Tests.
- X-Cycles. Full independent graph/link module: `struct CandidateGraph<Base>`, `struct Edge<Base>`, `enum LinkType`, `struct AlternatingPath<Base>`, `fn build_candidate_graph`, `fn add_links_for_rows/columns/blocks/unit`, `fn find_cycles_dfs`, own `fn sees` / `fn sees_both`.

### origin/copilot/implement-unique-rectangles-strategy
- 3 files (smallest, no new snapshots), +733/-1. Adds `impls/unique_rectangles.rs` (plural); modifies `impls/mod.rs`, `strategy_enum.rs`.
- Unique Rectangle Types 1, 2, 4: `struct UniqueRectangles`, `try_type_1/2/4`, `find_unique_rectangles`. Thin test coverage (no snapshot files).

### origin/copilot/implement-new-bug-strategy
- 6 files, +293/-3 (smallest strategy branch). Adds `impls/bug.rs`; modifies `impls/mod.rs`, `strategy_enum.rs`.
- BUG (Bivalue Universal Grave); minimal helper `count_candidate_in_group`. Also touches TS bindings/web constants for the new strategy.

### origin/copilot/add-chute-remote-pairs-strategy
- 17 files, +1429/-3. Adds `impls/chute_remote_pairs.rs` + snapshot set; modifies `impls/mod.rs`, `strategy_enum.rs`. Tests.
- Chute Remote Pairs via bipartite graph coloring: `fn find_bivalve_cells` (typo for "bivalue"), `fn group_by_candidates`, `fn build_adjacency`, `fn bipartite_coloring`, `fn collect_component`, own sees-helper `fn positions_see_each_other`.

### origin/copilot/add-rectangle-elimination-strategy
- 16 files, +1363/-3. Adds `impls/unique_rectangle.rs` (**singular**) + snapshot set; modifies `impls/mod.rs`, `strategy_enum.rs`.
- Unique Rectangle Types 1 and 2 only: `struct UniqueRectangle`, `try_type1/2`, `find_unique_rectangles`, own sees-helper `fn shares_group`.
- **Direct duplicate strategy** of `implement-unique-rectangles-strategy` (which also has Type 4). Near-identical but distinct file/type names; both register in `strategy_enum.rs`.

### origin/copilot/add-persistent-group-availability
- 6 files, +442/-252 (refactor, not a new strategy). Adds `strategic/group_candidate_availability.rs`; modifies `strategic/mod.rs`, `impls/group_intersection/mod.rs`, `impls/x_wing.rs`, `strategies/mod.rs`.
- **Infrastructure**: persistent `GroupCandidateAvailability<Base>` (`rows`, `columns`, `row_major_blocks`, `column_major_blocks`, each `CandidatesGroup<Base>`) as formal transposed view (candidate → group → index); refactors `XWing` + `GroupIntersection` onto it. Own `enum Axis { Row, Column }`.

### origin/copilot/evaluate-feature-gaps — OUT OF SCOPE
- Pure docs: 9 files, +714/0, all under `docs/feature-requests/` (product/UX tickets). No solver content. Predates the strategy branches' content ("9 solving strategies" in its README).

### origin/copilot/optimize-sat-pruning
- 2 files, +277/-17: `generator/mod.rs`, `solver/sat/mod.rs`.
- Adds `struct AmbiguousSolutionChecker<Base>` to `solver/sat/mod.rs`, wired into generator pruning — SAT-based check to detect ambiguous (multi-solution) candidate removals more cheaply during generation. Generator/SAT infra, not a strategy.

## Duplicated infrastructure across branches (key finding)

1. **"Two positions see each other" helper** — reimplemented under 6 names, no shared helper exists pre-branch (verified via `git grep`): `shares_house` (y_wing), `sees_each_other` (xyz_wing), `cells_see_each_other` (w_wing), `sees`/`sees_both` (x_cycles), `positions_see_each_other` (chute_remote_pairs), `shares_group` (unique_rectangle); plus `sees_any_of` (simple_colouring, slice variant).
2. **Strong-link / chain / graph structures** — 3 incompatible builds of the same primitive: w-wing (`StrongLink`/`StrongLinksMap`/`compute_strong_links`), simple-colouring (`Color`/`build_color_chains`/`find_all_strong_links`), x-cycles (`CandidateGraph`/`Edge`/`LinkType`/`AlternatingPath`).
3. **Bivalue-cell detection** — ad hoc in y_wing, xyz_wing, chute_remote_pairs.
4. **Transposed group-candidate-availability** — `add-persistent-group-availability` formalizes it persistently; swordfish independently reinvented a private non-persistent subset (`GroupCandidateIndexes`, rows/columns only) + its own `Axis` enum. Swordfish is a textbook consumer of the persistent abstraction.
5. **Whole-strategy duplicate** — the two UR branches implement the same technique under different names; cannot both merge (naming collision + redundancy).
