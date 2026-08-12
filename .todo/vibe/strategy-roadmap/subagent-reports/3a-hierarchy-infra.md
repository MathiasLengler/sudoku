# Phase 3a — Strategy Hierarchy & Shared-Infrastructure Proposals

Synthesis of `.tmp/0-*`, `.tmp/1-theory-spine.md`, `.tmp/2-*`. This is a map and a
proposal list — no roadmap, no implementations. Signatures/shapes/prose only. Every
theory claim traces to the spine (§ cited); `UNKNOWN` markers are preserved verbatim, not
back-filled.

Primitive vocabulary is fixed in Part (b) below; Part (a)'s `requires_primitives` fields
draw exclusively from it.

---

## Part (a) — Strategy hierarchy / dependency map

### The shape of the tree

The natural reading of the evidence is a **three-band tree rooted in one shared substrate**.

**Band 0 — the substrate everything reads from.** Every strategy on `dev` above Naked
Singles already answers one of two questions per grid step: "which cells are down to a
small candidate set?" (per-cell view) or "where can candidate *v* still go inside this
unit?" (transposed / group view). Hidden Singles, Naked/Locked Sets, Group Intersection
and X-Wing are all the *group* view; Naked Singles and every Bent-Set wing are the
*per-cell* view. Nothing on `dev` shares either view — they are re-derived per strategy
per `execute()` (baseline §"Transposed candidate structures": two near-identical
`GroupCandidateIndexes` on `dev`, plus a third reinvented on the Swordfish branch). This is
the substrate Part (b) proposes to name.

**Band 1 — Basics (already on `dev`, complete).** Naked Singles → Hidden Singles → Naked
Candidates (Pairs/Triples/Quads) → Locked Sets (naked+hidden via `v2` transpose) →
Intersection Removal (the three `GroupIntersection` variants). Spine §3 groups all of these
under "Basic Strategies"; the grading page (spine §2) gives them the density-scaled scores
`F`, `F×2`, `5×F`. This band is done and needs no branch work — the only branch touching it
(`add-locked-sets-reasoning`) merely populates the pre-existing empty `Reason::Candidates`
payload on Locked Sets (misc report §3: ~95% snapshot churn, sound, worth hand-porting; it
is *not* a new node).

**Band 2 — Tough tier, the natural next tier.** This is where the 14 branches cluster and
where the tree wants to grow next. It splits cleanly into three families that the spine and
the branches agree on:

- **Fish (Chaining family, X-Wing lineage).** X-Wing (on `dev`, n=2) → Swordfish (branch,
  n=3) → Jellyfish (spine, n=4, unbuilt). The Swordfish branch's own struct is a
  *byte-for-byte copy* of `x_wing.rs`'s private index (fish report §Fish-family
  cross-comparison), and the spine states outright that "All Swordfishes will break down
  into X-Wings" (spine §Swordfish) — i.e. sudokuwiki treats this as one pattern
  parameterised by *n*. This is the single clearest "one primitive unlocks a whole
  sub-tree" case in the evidence: a generic `fish(n)` scan turns three strategies into one.

- **Bent Sets.** The spine's family order (spine §3) is Chute Remote Pairs → Y-Wing →
  W-Wing → XYZ-Wing (with Rectangle Elimination positioned between Y-Wing and Swordfish).
  Branches attempted Y-Wing, XYZ-Wing, W-Wing (all slop-correct for their base case) and
  *mislabelled* Chute Remote Pairs (built generic Remote Pairs instead — see Band 3). XYZ
  explicitly "extends the Y-Wing pattern" (spine §XYZ-Wing) and W-Wing is "positioned after
  Y-Wings" (spine §W-Wing), so Y-Wing is the local root of this family. Every member of this
  family consumes the same two primitives: a bivalue-cell index and the `sees` predicate
  (W-Wing additionally needs strong links).

- **Colouring / single-digit chains (Chaining family).** Simple Colouring (branch,
  strong-link 2-colouring) → X-Cycles (branch, strong+weak alternating cycles), which the
  spine ties together directly: "X-Cycles are strongly related to Simple Coloring" and sits
  it in the Diabolical tier just above (spine §X-Cycles). Both branches hand-roll the same
  conjugate-pair scan and both reinvent a link graph (chains report §cross-branch). This
  family is the entry point to Band 3.

**Band 3 — Uniqueness + Diabolical (partially attempted, mostly future).** Unique
Rectangles (two duplicate branches, Types 1/2 sound, Type 4 defective) and BUG (branch,
sound) form the Uniqueness family — both rest on the unchecked single-solution axiom (UR
report §"Uniqueness assumption"; misc §BUG) and are the natural consumers of the SAT
uniqueness oracle (Part b, primitive 6). The Diabolical continuation is X-Cycles (branch,
defective/off-by-default) and, per the spine, XY-Chains — which the spine flags as
equivalent in power to BUG ("every instance of BUG can be solved by an XY-Chain", spine
§BUG) and which is the (Position,Value)-node generalisation of the colouring graph (chains
report §v2).

**Band 4 — Backend.** Brute Force (on `dev`) delegates to the introspective SAT/backtracking
solver; it is the escape hatch, not a human strategy, and sits at score 1e6.

### The one dependency that crosses the whole tree

Two structural primitives thread through every band: the **transposed group-availability
view** (Bands 0–2: Hidden Singles, Locked Sets, Group Intersection, X-Wing, Swordfish,
Jellyfish, and — as the substrate strong links are *derived* from — the entire colouring
family) and the **candidate link graph** (Band 2 W-Wing + colouring, Band 3 X-Cycles,
Rectangle Elimination, XY-Chains). The link graph is best understood as *layered on top of*
group-availability: a strong link is exactly "a group whose candidate-position count is 2",
which group-availability already answers with an O(1) popcount (infra report §"Chains").
That layering relationship is the backbone of the dependency map.

### Machine-readable map

Ordered by sudokuwiki solve/grading order (spine §3 family order + §"Tier/position"
column). `grade` uses the spine's per-page tier label or point value where stated, else
`UNKNOWN`. `requires_primitives` / `builds_on` use only nodes and primitives defined here;
every `builds_on` target is a node in this list.

```yaml
- strategy: NakedSingles
  family: Basic
  grade: F                       # spine §2 grading table (density factor)
  status: implemented
  requires_primitives: [single_candidate_cell]
  builds_on: []
  example_urls: [https://www.sudokuwiki.org/Getting_Started]
  slop_notes: ""

- strategy: HiddenSingles
  family: Basic
  grade: F×2                     # spine §2
  status: implemented
  requires_primitives: [group_candidate_availability]
  builds_on: [NakedSingles]      # spine: prerequisite UNKNOWN; ordering per §3 index
  example_urls: [https://www.sudokuwiki.org/Getting_Started]   # no standalone page (404); folded into Getting_Started
  slop_notes: ""

- strategy: NakedPairs
  family: Basic
  grade: 5×F                     # spine §2 (Naked Pair); Triples/Quads UNKNOWN
  status: implemented
  requires_primitives: [bivalue_cell]
  builds_on: [NakedSingles]      # spine §Naked Candidates: "beyond naked singles"
  example_urls: [https://www.sudokuwiki.org/Naked_Candidates]
  slop_notes: ""

- strategy: LockedSets
  family: Basic
  grade: UNKNOWN                 # spine §Hidden Candidates: no numeric points given
  status: implemented
  requires_primitives: [group_candidate_availability, nvalue_cell]
  builds_on: [NakedPairs]
  example_urls: [https://www.sudokuwiki.org/Hidden_Candidates, https://www.sudokuwiki.org/Naked_Candidates]
  slop_notes: "add-locked-sets-reasoning branch fills the empty Reason::Candidates payload (sound, ~95% snapshot churn, worth hand-porting); not a new node"

- strategy: GroupIntersection
  family: Basic
  grade: UNKNOWN                 # spine §Intersection Removal: "Basic", no numeric score
  status: implemented
  requires_primitives: [group_candidate_availability]
  builds_on: [NakedPairs]        # spine: prerequisite UNKNOWN
  example_urls: [https://www.sudokuwiki.org/Intersection_Removal]
  slop_notes: "on dev owns one of the two duplicate GroupCandidateIndexes (all 4 axes)"

- strategy: ChuteRemotePairs
  family: BentSets
  grade: UNKNOWN                 # spine §Chute Remote Pairs: exemplars 6.5-8.5 (pre-Oct-2025 scale); no point value
  status: absent
  requires_primitives: [bivalue_cell, chute_analysis, sees_predicate]
  builds_on: [XWing]             # spine: "should follow X-Wing instruction and precedes Y-Wings"
  example_urls: [https://www.sudokuwiki.org/Chute_Remote_Pairs]
  slop_notes: "add-chute-remote-pairs branch built generic Remote Pairs instead (see RemotePairs node); the real chute technique was never written — needs bivalue index + chute check, NO graph machinery"

- strategy: XWing
  family: Fish
  grade: 30                      # spine §2 (fixed); own page "Tough", exemplars 74-153 pre-Oct scale
  status: implemented
  requires_primitives: [group_candidate_availability, strong_link, fish_scan]
  builds_on: [GroupIntersection] # spine: assumes "locked pair" understanding; no named prereq
  example_urls: [https://www.sudokuwiki.org/X_Wing_Strategy]
  slop_notes: "on dev owns the second duplicate GroupCandidateIndexes (rows/columns only), copy-pasted verbatim by the Swordfish branch"

- strategy: YWing
  family: BentSets
  grade: Tough                   # spine §Y-Wing: "Tough", exemplars 6.1-7.6
  status: slop-correct
  requires_primitives: [bivalue_cell, sees_predicate]
  builds_on: [XWing]             # spine: explicit prereq UNKNOWN; Chute precedes it, RE follows it
  example_urls: [https://www.sudokuwiki.org/Y_Wing_Strategy]
  slop_notes: "shares_house helper returns true for pos==pos (no self-guard); weakest tests of the 3 wings — no assert_deductions, no snapshot macro, no sudokuwiki example puzzle"

- strategy: WWing
  family: BentSets
  grade: Tough                   # spine §W-Wing: "Tough", "positioned after Y-Wings"
  status: slop-correct
  requires_primitives: [bivalue_cell, strong_link, sees_predicate]
  builds_on: [YWing]             # spine: explicit
  example_urls: [https://www.sudokuwiki.org/W_Wing_Strategy]
  slop_notes: "single-W-Wing case only (remote-pair-chain & split-double variants absent); own StrongLink/StrongLinksMap can emit a pair twice (row+block); no snapshot macro, no sudokuwiki example; dead #[allow(dead_code)] test builder"

- strategy: XyzWing
  family: BentSets
  grade: UNKNOWN                 # spine §XYZ-Wing: explicit tier UNKNOWN
  status: slop-correct
  requires_primitives: [bivalue_cell, nvalue_cell, sees_predicate]
  builds_on: [YWing]             # spine: "extends the Y-Wing pattern by adding an extra candidate to the hinge"
  example_urls: [https://www.sudokuwiki.org/XYZ_Wing]
  slop_notes: "does NOT enforce a 'same chute' constraint the spine's prose mentions — flagged UNCERTAIN (spine ambiguous whether chute is a real precondition or incidental); reuses dev's all_candidates_positions (good); has snapshot macro but no sudokuwiki-example assert"

- strategy: RectangleElimination
  family: Fish                   # spine §Rectangle Elimination: index=Chaining; own page labels tier "Tough"
  grade: Tough                   # spine: "between Y-Wing and Swordfish"; ~35,826 instances/23,885 puzzles
  status: absent
  requires_primitives: [group_candidate_availability, strong_link, weak_link, chute_analysis, sees_predicate]
  builds_on: [YWing]             # spine: positioned between Y-Wing and Swordfish
  example_urls: [https://www.sudokuwiki.org/Rectangle_Elimination]
  slop_notes: "the branch NAMED add-rectangle-elimination actually built Unique Rectangles (see UniqueRectangles node); the real hinge/strong-link/weak-wing technique was never written"

- strategy: Swordfish
  family: Fish
  grade: Tough                   # spine §Swordfish: "Tough", exemplars 6.3-7.9; grading points UNKNOWN
  status: slop-correct
  requires_primitives: [group_candidate_availability, fish_scan]
  builds_on: [XWing]             # spine: "All Swordfishes will break down into X-Wings"
  example_urls: [https://www.sudokuwiki.org/Sword_Fish_Strategy]   # spine: worked example detail UNKNOWN (not captured)
  slop_notes: "logic sound (2-or-3 per line, union==3); GroupCandidateIndexes + Axis enum are a VERBATIM copy of x_wing.rs; tests 100% synthetic loose-invariant (no assert_deductions, no snapshot macro, no reference puzzle) — a wrong axis boundary would pass"

- strategy: Jellyfish
  family: Fish
  grade: UNKNOWN                 # spine §3 family table lists it after Swordfish; no page fetched
  status: absent
  requires_primitives: [group_candidate_availability, fish_scan]
  builds_on: [Swordfish]         # spine: X-Wing Family, ...Swordfish, Jellyfish (n=4)
  example_urls: []               # not fetched
  slop_notes: "unattempted; drops out for free as fish_scan(n=4)"

- strategy: SimpleColouring
  family: Colouring
  grade: Tough                   # spine §Simple Colouring: "Tough, between basic and X-Cycles"
  status: slop-correct
  requires_primitives: [strong_link, candidate_link_graph, sees_predicate]
  builds_on: [XWing]             # spine: prereq UNKNOWN; distinguished from Multi-Colouring/3D Medusa
  example_urls: [https://www.sudokuwiki.org/Simple_Colouring]      # spine: worked example UNKNOWN (not captured)
  slop_notes: "Rules 2 & 4 correct; Rule 7 ('one colour empties a unit') OMITTED (spine documents it, not UNKNOWN); one-Deduction-per-chain-per-call; own sees_any_of; hand-rolled 3x conjugate scan; fails template legs 2 & 3 (no snapshot macro, no reference puzzle)"

- strategy: UniqueRectangles
  family: Uniqueness
  grade: UNKNOWN                 # spine §Unique Rectangles: numeric tier UNKNOWN
  status: slop-defective
  requires_primitives: [bivalue_cell, uniqueness_oracle, sees_predicate]
  builds_on: []                  # spine: external prereq UNKNOWN; internal type-ordering only
  example_urls: [https://www.sudokuwiki.org/Unique_Rectangles]     # spine: several type coords UNKNOWN (not captured)
  slop_notes: "TWO duplicate branches (implement-unique-rectangles = Types 1/2/4; add-rectangle-elimination = Types 1/2, MISNAMED). Branch B is the better skeleton (idiomatic Deduction::try_from_iters, assert_deductions test, snapshot macro, sounder 'sees both via any unit' Type 2). Branch A's Type 4 floor-side branch is likely UNSOUND and untested — port roof-side only. Both rest on unchecked uniqueness axiom"

- strategy: BUG
  family: Uniqueness
  grade: 3.2-5.3                  # spine §BUG: puzzles requiring it score 3.2-5.3
  status: slop-correct
  requires_primitives: [bivalue_cell, group_candidate_availability, uniqueness_oracle]
  builds_on: []                   # spine: prereq UNKNOWN; "every instance of BUG can be solved by an XY-Chain"
  example_urls: [https://www.sudokuwiki.org/BUG]
  slop_notes: "correct BUG+1 rule; unaudited edge — returns first candidate if >1 satisfies count==3, no ambiguity guard; smallest/cleanest branch but below template (no reference-puzzle test, NO snapshot macro); relies on uniqueness axiom (no pure-elimination equivalent)"

- strategy: RemotePairs
  family: Chaining               # spine §3: listed under Chaining AND Deprecated Strategies
  grade: UNKNOWN
  status: slop-mislabeled
  requires_primitives: [bivalue_cell, candidate_link_graph, sees_predicate]
  builds_on: [SimpleColouring]   # same bipartite-2-colouring shape, generalised to remote pairs
  example_urls: [https://www.sudokuwiki.org/Remote_Pairs]
  slop_notes: "this is what add-chute-remote-pairs ACTUALLY built (its own UI labels it 'Remote Pairs' + /Remote_Pairs). Generic bipartite-coloured remote-pair chains; spine lists plain Remote Pairs as DEPRECATED. Bipartite/BFS machinery is over-engineering vs the assigned Chute strategy but correct for what it is. No reference puzzle. Either rename/rescope honestly or discard"

- strategy: XCycles
  family: Chaining
  grade: Diabolical              # spine §X-Cycles: "Diabolical"
  status: slop-defective
  requires_primitives: [strong_link, weak_link, candidate_link_graph, sees_predicate]
  builds_on: [SimpleColouring]   # spine: "X-Cycles are strongly related to Simple Coloring"
  example_urls: [https://www.sudokuwiki.org/X_Cycles]              # Part 2 (discontinuous loops) UNKNOWN (not fetched)
  slop_notes: "continuous-loop Rule 1 matches spine; discontinuous-loop code adds an extra last_link constraint of unclear justification — UNCERTAIN, unverifiable (Part 2 unfetched). Concrete defect: dedups cycles by sorted node-SET not edge-sequence, silently dropping distinct cycles. AlternatingPath::extend clones O(len^2). OFF by default (not in default_solver_strategies)"

- strategy: XYChains
  family: Chaining
  grade: UNKNOWN                 # spine §3 lists under Chaining; grading points UNKNOWN
  status: absent
  requires_primitives: [bivalue_cell, candidate_link_graph, sees_predicate]
  builds_on: [SimpleColouring]   # spine: equivalent power to BUG; (Position,Value)-node generalisation
  example_urls: []               # no dedicated page fetched
  slop_notes: "unattempted; needs the v2 (Position,Value)-node link graph (chains report §v2). Placed because spine ties it to BUG and it is the diabolical continuation of the colouring family"

- strategy: BruteForce
  family: Backend
  grade: n/a                     # solver escape hatch, ad-hoc score 1e6 on dev
  status: implemented
  requires_primitives: [uniqueness_oracle]
  builds_on: []
  example_urls: []
  slop_notes: "delegates to introspective SAT/backtracking solver; not a human strategy"
```

Node accounting: every `dev` strategy (NakedSingles, HiddenSingles, NakedPairs, LockedSets,
GroupIntersection, XWing, BruteForce) and every branch strategy (YWing, XyzWing, WWing,
Swordfish, SimpleColouring, XCycles, UniqueRectangles, BUG, RemotePairs [what chute built],
ChuteRemotePairs [what it should have built]) appears exactly once, plus the near-term spine
nodes (Jellyfish, RectangleElimination, XYChains). Locked-Sets-reasoning is a modification of
the LockedSets node, not a separate node (recorded in its `slop_notes`).

---

## Part (b) — Shared-infrastructure proposals

Six candidate primitives, ordered by leverage (present consumers + future spine strategies
unlocked). Each names a single home in this codebase, gives shapes/signatures only, and
lists the design questions the owner should decide himself. The two structural primitives
(#1, #2) plus the trivial-but-ubiquitous #3 account for essentially all the duplication the
survey found.

### 1. `GroupCandidateAvailability` — persistent transposed availability

**Evidence.** Duplicated *on `dev`* twice already (baseline §"Transposed candidate
structures"): `GroupCandidateIndexes` in `group_intersection/mod.rs:127-175` (all four axes)
and in `x_wing.rs:221-259` (rows/columns only), each rebuilt every `execute()` via a full
grid scan. The Swordfish branch reinvented a *third* copy — fish report §Fish-family
cross-comparison confirms it is "byte-for-byte identical" to `x_wing.rs`'s, down to variable
names, plus its own duplicate `Axis` enum. The `add-persistent-group-availability` branch
already formalised the persistent version (infra report §1) — so a reference implementation
of the *struct* exists; what's unresolved is persistence/incremental wiring (below).

**Home + shape.** `sudoku-rs/src/solver/strategic/group_candidate_availability.rs` (the
branch's chosen location). Shapes as the branch defined them:
```
enum Axis { Row, Column }
    fn other(self) -> Self
    fn coordinates_to_pos<Base>(self, axis_coord, other_axis_coord) -> Position<Base>
struct GroupCandidateAvailability<Base> { rows, columns, row_major_blocks, column_major_blocks }  // each CandidatesGroup<Base>
    fn axis(&self, Axis) -> &CandidatesGroup<Base>
    fn insert(&mut self, pos: Position<Base>)
    fn delete(&mut self, pos: Position<Base>)
struct StrategicGroupAvailability<Base> { candidates: Vec<GroupCandidateAvailability<Base>> }  // one per Value
    fn from_grid(&Grid<Base>) -> Self
    fn get(&self, Value<Base>) -> &GroupCandidateAvailability<Base>
    fn iter(&self) -> impl Iterator<Item=(Value<Base>, &GroupCandidateAvailability<Base>)>
```
Threaded via the branch's additive `Strategy::execute_with_availability(self, grid, &avail)`
trait method (default forwards to `execute`), so per-strategy adoption is opt-in and
non-breaking.

**Consumes now / unlocks.** Now: HiddenSingles, LockedSets, GroupIntersection, XWing
(baseline shows these are the group-view strategies; the branch already refactored XWing +
GroupIntersection onto it). Unlocks: **Swordfish** (its `rows`/`columns` fields *are* the
duplicated struct — infra report §"Fish strategies"), **Jellyfish**, and — critically — it
is the substrate the **candidate link graph** (#4) derives strong/weak links from: a strong
link is a group with `count()==2`, an O(1) popcount on this bitset (infra report §"Chains").
BUG's per-group candidate counting (misc §BUG) also reads naturally from it. Leverage ≈ 8
strategies plus the entire link-graph tier layered above.

**Open design questions for the owner.**
- **Persistent-incremental vs rebuild-per-step.** The branch left this half unbuilt: the
  incremental API (`insert`/`delete`/`delete_candidate`/`delete_all_candidates`/`set_value`)
  is fully written and unit-tested but has *zero call sites* outside its own test module
  (infra report §1, §3) — it is never invoked from `Deduction::apply`/`Action::apply`, and
  the solver still calls `from_grid()` (full rescan) on every step because the human-style
  loop restarts from the top after each deduction. Decide: wire incremental updates into the
  deduction-application path and hold the struct as persistent `Solver`/`SolverPathIter`
  state, **or** drop the incremental API until a benchmark justifies it.
- **Benchmark gap.** The branch's two criterion benchmarks time `from_grid` in isolation with
  no comparison against per-strategy rebuild cost and no full-solve-loop measurement (infra
  §3) — so "is construction actually a bottleneck?" is instrumented but unanswered. The owner
  should decide the benchmark shape before committing to persistence.
- **Block-position resolution seam.** The new module has no `Axis::coordinates_to_pos` for
  blocks; block-index-to-`Position` still goes through the older, separate
  `position::BlockSegment::block_position`/`axis_position` API (infra §Answering, "Fish"). A
  chain-builder reading block strong links must bridge that seam.

### 2. `Position::sees` predicate

**Evidence.** The single most duplicated primitive: reimplemented under **7 names** across
branches with *no* shared helper on `dev` (survey §Duplicated infra #1): `shares_house`
(y_wing), `sees_each_other` (xyz_wing), `cells_see_each_other` (w_wing), `sees`/`sees_both`
(x_cycles), `positions_see_each_other` (chute/remote), `shares_group` (unique_rectangle),
`sees_any_of` (simple_colouring, slice variant) — plus an 8th, *inlined and unnamed*, in the
`implement-unique-rectangles` branch (UR report §2). Worse, they disagree on a real edge case
(wings report §cross-branch): y_wing and xyz_wing return `true` for `pos == pos` (no
self-guard), masked only by external call-site guards that a refactor could silently drop;
w_wing guards correctly.

**Home + shape.** `sudoku-rs/src/position/mod.rs`, alongside the `to_row`/`to_column`/
`to_block` accessors it composes:
```
impl<Base: SudokuBase> Position<Base> {
    fn sees(self, other: Self) -> bool   // self-check baked in (pos.sees(pos) == false)
}
```
The `sees_both`/`sees_any_of` variants in the branches are trivial folds over this and need
no home of their own (wings report §cross-branch).

**Consumes now / unlocks.** Highest raw consumer count in the tree: every Bent-Set wing
(Y-Wing, XYZ-Wing, W-Wing, Chute Remote Pairs, Rectangle Elimination), every colouring/chain
strategy (Simple Colouring, X-Cycles, Remote Pairs, XY-Chains) and Unique Rectangles — ~10
strategies. Lowest design risk of any primitive here (a 3-line predicate); the leverage is in
correctness-consolidation (one self-guard, not eight) rather than structural unlock.

**Open design questions.** Only one, and minor: confirm the self-guard semantics
(`pos.sees(pos) == false`) matches every current call site's expectation before deleting the
local copies — w_wing already assumes `false`, the two wings assume the external guard
handles it.

### 3. Bivalue / candidate-count cell index

**Evidence.** Ad-hoc per-cell candidate-count scans reinvented across at least four branches
(survey #3; wings §cross-branch): y_wing (count==2), xyz_wing (count==2 *and* ==3, bucketed
in one pass), the chute/remote branch (`find_bivalve_cells` — sic), and both UR branches.
`dev` already has the building block `Grid::all_candidates_positions()`
(`grid/mod.rs:729`) that xyz_wing correctly reuses (wings §2) — the missing piece is the
count filter on top of it. Distinct from #1: this is *position-indexed by candidate-count*,
the orthogonal per-cell view, which group-availability explicitly does **not** serve (infra
§Answering, "Wings").

**Home + shape.** `sudoku-rs/src/grid/mod.rs`, next to `all_candidates_positions()`:
```
impl<Base: SudokuBase> Grid<Base> {
    fn positions_with_candidate_count(&self, count: u8) -> Vec<(Position<Base>, Candidates<Base>)>
}
```
Count-parameterised, not bivalue-specific, because XYZ-Wing already needs trivalue and BUG
needs "exactly one trivalue among all-bivalue" (misc §BUG). A `bivalue_positions()` wrapper
(`count == 2`) can sit on top for call-site readability (wings §Bivalue).

**Consumes now / unlocks.** Now: Y-Wing, XYZ-Wing, W-Wing, both UR branches, Remote Pairs,
BUG. Unlocks: Chute Remote Pairs, XY-Chains. Leverage ≈ 8, and it is the per-cell counterpart
to #1 — together they cover both grid views.

**Open design questions.** Whether a returned `Vec` (allocating) or a lazy `impl Iterator`
better fits the hot solve loop — the wings report proposes `Vec`; a rebuild-per-step concern
mirrors #1's persistence question if this ever becomes hot.

### 4. Candidate link graph (strong / weak links)

**Evidence.** Three *incompatible* builds of the same concept (survey #2; chains report
§cross-branch reconciliation): W-Wing's flat `StrongLink`/`StrongLinksMap` (strong only,
all-candidates-at-once, no traversal), Simple Colouring's `Color`/adjacency-map 2-colouring
(strong only, per-candidate, connected-components walk), and X-Cycles' full
`CandidateGraph`/`Edge`/`LinkType`/`AlternatingPath` (strong+weak, per-candidate, alternating
cycle DFS). Remote Pairs adds a fourth bipartite-colouring variant (misc §2). All four
hand-roll the row/column/block conjugate scan that #1 already answers.

**Home + shape.** New module under `solver/strategic/` (chains report §"Proposed
abstraction"), layered on #1 so links are *derived* from group-availability counts, not
rescanned:
```
enum LinkKind { Strong, Weak }
struct Link<Base> { to: Position<Base>, kind: LinkKind }
struct CandidateLinkGraph<Base> { adjacency: CandidateMap<Base, BTreeMap<Position<Base>, Vec<Link<Base>>>> }
    fn strong_links(&self, candidate: Value<Base>) -> impl Iterator<Item=(Position<Base>, Position<Base>)>   // flat view — serves W-Wing, no walk
    fn neighbors(&self, candidate: Value<Base>, pos: Position<Base>, kind: Option<LinkKind>) -> impl Iterator<Item=Position<Base>>
```
`CandidateMap<Base, T>` follows the existing dense-by-key `StrategyMap`/`PositionMap`
convention. Component-colouring and cycle-search stay *outside* the struct as strategy-local
free functions over `neighbors()` — the struct is a passive index, mirroring how
group-availability is consumed (chains §"Proposed abstraction").

**Consumes now / unlocks.** Now: W-Wing (`strong_links` flat view only, no graph walk),
Simple Colouring (`neighbors(Strong)` + external 2-colouring), X-Cycles (`neighbors(None)` +
external alternating DFS), Remote Pairs. Unlocks the whole Diabolical/Chaining tier:
Rectangle Elimination (strong hinge + weak wing), XY-Chains, and per the spine's family list
3D Medusa and Multi-Colouring. Highest *structural* leverage in the tree — it is the gateway
to Band 3.

**Open design questions for the owner.**
- **`Position` nodes (v1) vs `(Position, Value)` nodes (v2).** Chains report §v1/§v2: v1
  (single-candidate-scoped, node=`Position`) unifies all three current branches and is the
  minimum. But 3D Medusa and XY-Chains need multi-candidate nodes; a `(Position,Value)`-keyed
  graph subsumes v1 as a fixed-second-coordinate slice. Decide up front whether to key on
  `(Position,Value)` from the start so v1's type doesn't need a breaking rename when those
  land.
- **Dedup contract.** W-Wing's builder can emit the same physical pair twice (once per unit
  type it is simultaneously the sole occupant of — wings §W-Wing gotcha 1). The canonical
  structure should either store links as a set or document that `strong_links` may yield
  duplicates (harmless for non-emptiness checks, wrong for anyone counting).
- **Persistence** inherits #1's rebuild-per-step question — if #1 stays rebuild-per-step, so
  does everything built on it.

### 5. Generic `fish(n)` scan

**Evidence.** X-Wing (n=2, `dev`) and Swordfish (n=3, branch) are the *same* algorithm; the
Swordfish branch's index struct is a verbatim copy of X-Wing's (fish report §Fish-family
cross-comparison), and the spine states the identity directly: "All Swordfishes will break
down into X-Wings" (spine §Swordfish), framing X-Wing→Swordfish→Jellyfish as one pattern
parameterised by *n*.

**Home + shape.** A free function / method over #1 (fish report §Fish-family): for a given
candidate and base axis, filter lines to `2..=n` candidate positions, enumerate
`C(lines, n)` combinations, keep those whose union of opposite-axis coordinates has size
exactly `n`, then eliminate the candidate on the covering lines outside the base lines.
Signature shape only:
```
fn fish_scan<Base>(avail: &GroupCandidateAvailability<Base>, candidate: Value<Base>, axis: Axis, n: usize) -> ...Deductions
```
(X-Wing's n=2 pairwise-equality check is the degenerate case where a 2-line union of size 2
forces identical pairs — fish report §"Is Swordfish a natural n=2 generalization".)

**Consumes now / unlocks.** X-Wing, Swordfish now; **Jellyfish** (n=4) falls out for free.
Leverage = 3 strategies collapsed into one implementation, entirely dependent on #1.

**Open design questions.** Whether to migrate the existing X-Wing off its bespoke pairwise
check onto the generic union-based scan (behaviour-equivalent but changes a working, tested
strategy) or leave X-Wing as-is and only use `fish(n)` for n≥3. Combinatorial cost of
`C(lines, n)` is fine at 9×9/16×16 (fish §3) but the owner should note it grows for Base5.

### 6. Uniqueness-oracle plumbing (SAT-backed) — defer details to the SAT stage

**Evidence.** Unique Rectangles (both branches) and BUG rest entirely on the unchecked
"published Sudokus have one solution" axiom — asserted, never verified (UR §"Uniqueness
assumption"; misc §BUG "no SAT-solver equivalent path"). The repo already has SAT
solution-enumeration (`solver/sat::SolverIter`, baseline §SAT) used only for
generation/full-solve today. The `optimize-sat-pruning` branch demonstrates the reusable
shape — a persistent `varisat::Solver` with incremental `assume()`+`solve()` — but the SAT
report flags the actual gap (sat report §4): the base `sat::Solver` fixes its assumption set
at construction and exposes **no** public re-`assume`/re-`solve` method, so
`AmbiguousSolutionChecker` had to reimplement incremental querying bespoke. There is also no
unsat-core extraction for explaining *why* a candidate is forced (sat §4, relevant to the
`Reason::Cell` TODO).

**Home + shape.** Placeholder only — full design belongs to the Phase-3b SAT-fit stage. Home
would be `solver/sat/`; the query shape a uniqueness strategy needs is "assume current
givens + one denial literal, `solve()`, read a boolean" (sat §4), which
`Solver::with_candidates_filter` already supports for the *single*-query case with no new
plumbing. The missing piece is the *incremental multi-query* generalisation (no known target
solution to exclude, just current givens + per-query deltas).

**Consumes now / unlocks.** Unique Rectangles, BUG (uniqueness precondition check); any
future Uniqueness-family strategy (Extended/Hidden/Avoidable Rectangles, BUG+1). Lowest
placement priority here because it is orthogonal to the strategy-graph work and its design is
deferred.

**Open design questions (flag, don't resolve — SAT stage owns these).** Whether to expose a
general incremental-assume/resolve capability on `sat::Solver`; whether UR/BUG should
actually *gate* on a uniqueness check or keep the axiom implicit (performance vs correctness
on malformed grids); whether to plumb unsat-core extraction for `Reason` payloads.

---

## Consistency notes

- Primitive tokens used in Part (a) YAML: `single_candidate_cell`, `bivalue_cell`,
  `nvalue_cell`, `group_candidate_availability`, `sees_predicate`, `strong_link`,
  `weak_link`, `candidate_link_graph`, `fish_scan`, `chute_analysis`, `uniqueness_oracle`.
  All except `single_candidate_cell`, `nvalue_cell`, and `chute_analysis` map to a Part (b)
  primitive; those three are noted here as vocabulary-only (no dedicated infra proposed —
  `single_candidate_cell` is Naked Singles' trivial endpoint; `nvalue_cell` is the
  count-parameterised form of #3; `chute_analysis` is a per-strategy geometric check the
  spine names for Bent Sets, not shared machinery any branch reinvented).
- Every `builds_on` target (`NakedSingles`, `NakedPairs`, `GroupIntersection`, `XWing`,
  `YWing`, `Swordfish`, `SimpleColouring`) is itself a node in the map.
```
