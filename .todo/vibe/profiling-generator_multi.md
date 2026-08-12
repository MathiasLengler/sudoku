# Profiling `generator_multi` — session report (2026-08-12)

Nothing here is decided. Options are recorded with attribution: **[M]** = Mathias, **[C]** = Claude
suggestion. No approach was approved.

Reproducer: `just profile-generator_multi`.

## TL;DR

The workload is ~96% the pruning loop, and inside it the strategic solver. The single largest item
is `Grid::is_directly_consistent` at **33% of cycles** — the per-step `validate()` in
`Solver::try_strategies`. It is not redundant [M]: solver deductions are exactly what can break
consistency, so it must be re-established each step; it is what turns an invalid user grid into a
clean `Err`. In the pruning path it provably never fires, but that argues for making it cheap, not
for gating it.

The cost is structural [M]: the strategic solver reconstructs the *value-space* view of the grid
from the *position-space* one on every query. `direct_candidates`, `Group::transpose` in LockedSets,
and the backtracking solver's `GroupAvailability` are three instances of the same transpose; only
the strategic solver rebuilds it each time.

This corroborates the substrate conclusion already reached independently in
[strategy-roadmap.md](strategy-roadmap/strategy-roadmap.md) (Band 0: transposed group view +
per-cell view, "rebuild-per-step first, benchmark before wiring incremental").

## Capture

AMD Ryzen 9 7950X, `release-debug-full`, `--call-graph fp`, 6.6 s, single-threaded (one rayon
worker; `parallel: false` in the bin defaults).

| event | count | derived |
| --- | --- | --- |
| cycles | 34.99 G | |
| branches | 19.43 G | 0.56 branches/cycle |
| branch-misses | 0.339 G | 1.75% miss rate |

0.56 branches/cycle is the headline shape: a branch roughly every other cycle. The miss *rate* is
healthy, so this is retired-branch volume, not misprediction — iterator state machines executing,
not stalling. Reduce work rather than improve predictability.

### Two capture gotchas (both cost real time)

1. `[build] rustflags` in `.cargo/config.toml` was silently ignored. `~/.cargo/config.toml` sets
   `target.x86_64-unknown-linux-gnu.rustflags` for the `wild` linker, and those sources are
   [mutually exclusive with `build.rustflags`](https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags) —
   target wins outright. `-Zthreads=8` had never applied to native builds either. Fixed by moving to
   `[target.'cfg(not(target_family = "wasm"))']`, which joins with the triple entry instead of
   replacing it. Verify with `jq .rustflags target/release-debug-full/.fingerprint/sudoku-*/bin-generator_multi.json`.
2. `--call-graph dwarf,8192` (hotspot's default) produced leaf-only stacks — the 8 KB window is too
   small for these call chains, and the resulting 670 MB capture yielded no caller attribution at
   all. Frame pointers cost ~1% wall-clock here and shrink perf.data to 8 MB.

### Methodology note

Self-time alone misattributed the top symbol. `direct_candidates` is 24% self, and reading its call
sites suggested `set_all_direct_candidates` as the caller. The actual split, once callchains worked,
is 20.74% via `is_directly_consistent` and 3.51% via `set_all_direct_candidates` — a 6:1 error that
inverted the ranking. Don't rank optimizations from a flat profile.

## Call tree (cycles, inclusive)

```
try_delete_cell_at_pos                     95.84%   (self 0.01%)
└─ is_solvable_with_strategies             93.70%
   └─ strategic::Solver                    88.41%
      ├─ is_directly_consistent            33.25%   (self  1.22%)   ← validate(), every step
      ├─ StrategyEnum::execute             38.96%
      │  ├─ LockedSets / find_locked_set   14.30%   (self 12.34%)
      │  ├─ GroupIntersection               4.13%
      │  ├─ NakedPairs                      1.52%
      │  └─ XWing                           1.46%
      ├─ Deductions::apply                  6.80%   (self  0.92%)
      └─ is_solved                          3.46%
```

`direct_candidates` — 25.34% inclusive / 24.38% self:

- 20.74% under `is_directly_consistent`
- 3.51% under `set_all_direct_candidates`

Cross-cutting, not a separate tree branch:

| | inclusive | self |
| --- | --- | --- |
| neighbour `FlatMap<Chain<Chain<…>>>` | 9.49% | 7.63% |
| `Vec<Position>` `SpecFromIter` | 6.54% | 2.72% |
| `ZipEq` in `Group::from_iter_checked` | 4.82% | 4.64% |
| `Deductions` BTreeSet/BTreeMap | — | ~5% summed |

The metric this run optimizes for (XWing deduction count) is 1.46%. Cost is the prune loop's
solvability checks, not metric evaluation.

## Options discussed

### A. Single-pass bitmask `is_directly_consistent` [C]

`is_directly_consistent` checks four properties. All four fall out of two linear passes over the
cells plus one pass over the groups, replacing ~2200 branchy neighbour-iterator steps and 27 group
scans:

| | property | current |
| --- | --- | --- |
| a | no empty candidates cell | free |
| b | `candidates(p) ⊆ direct_candidates(p)` ∀p | 81 × 27 neighbour walks |
| c | no duplicate value in group | 27 group scans |
| d | every value in group as value-or-candidate | 27 group scans |

Both accumulators are `GroupAvailability`, i.e. a `CandidatesGroup<Base>` per group type — the
existing set abstraction, no raw bit twiddling:

```rust
// pass 1: one walk over all cells
let mut availability = GroupAvailability::all(); // values still placeable per group
let mut seen = GroupAvailability::new();         // value-or-candidate union per group

for (index, cell) in cells {
    match *cell.state() {
        Value(value) | FixedValue(value) => {
            ensure!(availability.has_in_each_group(index, value)); // (c) already placed
            availability.delete(index, value);
            seen.insert(index, value);
        }
        Candidates(candidates) => {
            ensure!(!candidates.is_empty());                       // (a)
            seen.union(index, candidates);
        }
    }
}

// pass 2: candidates cells only
ensure!(candidates.without(availability.available_candidates_at(index)).is_empty()); // (b)

// (d)
seen.iter().all(Candidates::is_full)
```

`has_in_each_group` and a `union` alongside the existing `insert`/`delete` are the only additions;
both follow the existing `mutate` pattern in
[group_availability.rs](../../sudoku-rs/src/solver/backtracking/group_availability.rs).

Pass 2 is `is_directly_consistent_at`'s existing expression with `direct_candidates(pos)` swapped for
`available_candidates_at(index)` — same semantics, and exact rather than approximate:
`direct_candidates(p)` is `ALL` minus the union of neighbour values, and `p` itself contributes
nothing to that union since it holds candidates rather than a value.

No API change, no new persistent state. Also subsumes option D: the availability built in pass 1 is
exactly what `set_all_direct_candidates` needs — and is exactly the structure option B would maintain
incrementally instead of rebuilding.

### B. Mirrored value-space view in the strategic solver [M]

Keep a [`GroupAvailability`](../../sudoku-rs/src/solver/backtracking/group_availability.rs)-like
structure alongside the grid and mirror writes from deduction application, as the backtracking
solver already does.

`available_candidates_at` is exactly `direct_candidates` in 3 loads + 2 ANDs. It is derived from
placed values only, so it covers (b) and turns (c) into a write-time assert, but not (a) or (d) —
those need actual candidates. Covering them needs a second structure, the transposed candidate view
(per group, per value, a coordinate bitmask) [C]:

```
availability:        3 × 9 × Candidates   =  54 bytes
candidate_positions: 27 × 9 × Coordinates = 486 bytes
```

Under 1 KB against a 324-byte grid; both stay L1-resident. Then (d) is
`positions[g][v] != 0 || !available[g].has(v)` and (a) is a flat 81-cell test.

Maintenance is bounded: `DeleteCandidates` is O(|cs|) bit-clears; `SetValue` is ≤60, the same
neighbour set `update_direct_candidates_for_new_value` already walks. Reads dwarf writes — every
strategy scans all 27 groups per step against 1–3 actions per deduction.

Readability [M is the constraint; the sketch is C]: `Strategy::execute` already takes `&Grid<Base>`,
so strategies are read-only. A `GridView` wrapper with `Deref<Target = Grid<Base>>` keeps every
existing strategy compiling verbatim and allows per-strategy migration. `LockedSets` would drop both
its group conversion and its `transpose()` (the FIXME at
[locked_sets.rs:42](../../sudoku-rs/src/solver/strategic/strategies/impls/locked_sets.rs#L42));
`HiddenSingles` becomes `positions[g][v].count() == 1`.

Risk [C]: derived state can drift from the grid, and drift yields wrong deductions with no error —
strictly worse than slow-but-correct. Mitigation: keep the full `is_directly_consistent` plus a full
recompute-and-compare of the view, both behind `debug_assert`, after each apply. The expensive check
becomes the oracle for the fast path. Two things help — strategies cannot mutate, so the write choke
point is just `Deduction::apply`; and `History<Grid<Base>>` stores whole-grid snapshots, so undo is a
rebuild, not an incremental inverse.

A and B are not exclusive: A is the recompute-from-scratch form of the data B maintains, so A doubles
as B's debug oracle rather than being thrown away.

### C. `LockedSets` — 14.30% inclusive, 12.34% self

12.34% is inside the combinatorial search over `potential_locked_set_indexes.combinations(set_size)`.
Option B makes feeding the search cheaper but not the search itself.

- Tree-pruning the combination enumeration — pre-existing TODO at
  [v2.rs:83](../../sudoku-rs/src/solver/strategic/strategies/impls/locked_sets/v2.rs#L83) [M]
- Dirty-group tracking so `execute` skips groups untouched by the last deduction [C]
- Pre-existing note at [locked_sets.rs:60](../../sudoku-rs/src/solver/strategic/strategies/impls/locked_sets.rs#L60):
  v1 beats v2 by up to 10× on small groups, v2 wins by ~20000× on large ones [M]

### D. Static neighbour/group position tables [C]

Pre-existing TODO at [grid/mod.rs:739](../../sudoku-rs/src/grid/mod.rs#L739) ("reimplement without
chain (VTune: bad speculation + unique version)") [M]. The neighbour iterator is 9.49% inclusive /
7.63% self, mostly from `Deductions::apply` → `update_direct_candidates_for_new_value`; `perf
annotate` shows positions recomputed arithmetically (`imul`, `shr`, `div_ceil`) per step. Const
`[[Position; 20]; 81]` / `[[Position; 9]; 27]` tables, following the existing
`base::cell_index_to_block_index::BASE_3` pattern, would also remove the `ZipEq` (4.64% self) and the
`Vec<Position>` allocations in `all_candidates_positions()`.

Largely subsumed by B for the strategic-solver paths.

### E. `Deduction` collections [C]

`BTreeMap<Position, _>` / `BTreeSet<Deduction>`, ~5% self summed plus node allocations. For ≤81 dense
`u16` keys a bitmask + flat array beats a B-tree. Lowest priority.

### F. Rejected

Hoisting or gating the per-step `validate()` [C, withdrawn]. Wrong on the merits [M]: consistency is
not preserved by the solver's own mutations, so it is not an invariant to verify once. A
`validate_each_step` flag was also considered and rejected [C]: the generator is the highest-volume
consumer, and gating it there means a strategy bug producing an inconsistent grid would go unnoticed
in exactly that path.

Incremental validation of only the cells and groups touched by the last deduction [C]: sound, but
`SetValue` transitively touches 20 neighbours and their groups, so for (d) the affected set
approaches the whole grid. Not worth the bookkeeping over A.

## Suggested sequencing [C]

1. A — contained, no new state, no API change, most of the 33%.
2. Re-profile. Removing 33% reshuffles everything below it; `direct_candidates` should fall from 24%
   to ~3.5%.
3. B — additionally reaches C's feed cost and D.
4. C's search pruning, then E.

## Open

- Where the remainder of `StrategyEnum::execute`'s 38.96% sits — LockedSets, GroupIntersection,
  NakedPairs and XWing account for ~21%; the rest is unattributed between the singles strategies and
  per-group collection.
- Whether `Deduction::validate` already covers (a) and (d), which determines whether A can also
  simplify the apply path.
- `just profile-generator_multi` carries a TODO to test more perf event types. `instructions` would
  give IPC; on Zen 4, frontend-bound vs dependency-stalled would need
  `de_no_dispatch_per_slot.no_ops_from_frontend`.
