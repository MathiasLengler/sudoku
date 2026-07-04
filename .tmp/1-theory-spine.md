# Sudoku Theory Spine (sudokuwiki.org)

Grounded exclusively in fetched sudokuwiki.org pages. Any fact not stated on a fetched page is marked `UNKNOWN`.

## 1. Source List

| # | URL | Fetch Status |
|---|-----|--------------|
| 1 | https://www.sudokuwiki.org/Strategy_Families | OK |
| 2 | https://www.sudokuwiki.org/Grading_Puzzles | OK |
| 3 | https://www.sudokuwiki.org/Getting_Started | OK |
| 4 | https://www.sudokuwiki.org/Hidden_Singles | 404 Not Found — no dedicated page exists. Confirmed via web search (site:sudokuwiki.org) that Hidden Singles content is covered on the `Getting_Started` page rather than a standalone page. Retried once (search-based cross-check) before accepting. |
| 5 | https://www.sudokuwiki.org/Naked_Candidates | OK |
| 6 | https://www.sudokuwiki.org/Hidden_Candidates | OK |
| 7 | https://www.sudokuwiki.org/Intersection_Removal | OK |
| 8 | https://www.sudokuwiki.org/X_Wing_Strategy | OK |
| 9 | https://www.sudokuwiki.org/Y_Wing_Strategy | OK |
| 10 | https://www.sudokuwiki.org/XYZ_Wing | OK |
| 11 | https://www.sudokuwiki.org/W_Wing_Strategy | OK |
| 12 | https://www.sudokuwiki.org/Sword_Fish_Strategy | OK |
| 13 | https://www.sudokuwiki.org/Simple_Colouring | OK |
| 14 | https://www.sudokuwiki.org/X_Cycles | OK (Part 1 only; Part 2 at `/X_Cycles_Part_2` was not fetched — discontinuous-loop detail beyond the Part 1 summary is `UNKNOWN (not fetched)`) |
| 15 | https://www.sudokuwiki.org/Unique_Rectangles | OK |
| 16 | https://www.sudokuwiki.org/BUG | OK |
| 17 | https://www.sudokuwiki.org/Chute_Remote_Pairs | OK |
| 18 | https://www.sudokuwiki.org/Rectangle_Elimination | OK |

17/18 fetches succeeded; the one failure (Hidden Singles) is a confirmed non-existent standalone page, not a transient error.

## 2. Grading Model Summary (from `/Grading_Puzzles`)

Per the page, the grading system was redesigned as of **October 1, 2025**.

**Difficulty tiers (9x9 standard Sudoku):**

| Grade | Score Range |
|-------|-------------|
| Kids | < 3 |
| Gentle | 3 to < 4 |
| Moderate | 4 to < 5 |
| Tough | 5 to < 7 |
| Diabolical | 7 to < 9 |
| Extreme | 9+ |

"Different puzzle variants (Sudoku X, Jigsaw, Killer, KenKen, etc.) have adjusted ranges." (exact adjusted ranges: `UNKNOWN`, page did not enumerate them in the fetched excerpt).

**Scoring methodology:**

- Step 1 — Candidate Density Factor: "the total number of candidates on the board at any one step" drives the assessment. Formula: `F = C / 727 × 20`, where `C` = current candidate count and `727` = total candidate slots on a 9x9 board.
- Step 2 — Strategy Point Values (base points, multiplied by F where noted):

  | Strategy | Points |
  |----------|--------|
  | Naked Singles | F |
  | Hidden Singles | F × 2 |
  | Naked Pair | 5 × F |
  | X-Wing | 30 |
  | Exocet | 300 |
  | Alternating Inference Chains | 100 + chain length |

  (Points for other strategies in this spine — Naked/Hidden Triples/Quads, Intersection Removal, Y-Wing, XYZ-Wing, W-Wing, Swordfish, Simple Colouring, X-Cycles, Unique Rectangles, BUG, Chute Remote Pairs, Rectangle Elimination — are `UNKNOWN`; the fetched excerpt of `/Grading_Puzzles` only gave the six rows above as examples.)

- Step 3 — Score Normalization: "the sum of the scores for each step is the puzzle score." Raw scores are normalized logarithmically: 9x9 puzzles use `Log₅(score) × 2`; 6x6 puzzles use `Log₄(score) × 2`. Produces the final 1–10+ scale rating.

**Design principle stated on the page:** the redesign dropped a previous "rounds-based" weighting system, because "if you have found a pattern then it doesn't really matter how many candidates are removed" — i.e., prolific eliminations are no longer penalized/inflated.

Note: this Oct-2025 model is distinct from the older informal "Basic / Tough / Diabolical / Extreme" tier labels that many individual strategy pages still use in their own text (see per-strategy entries below) — both are reported since both come from fetched sudokuwiki content, but they are not necessarily the same scale.

## 3. Strategy Order / Family Table

From `/Strategy_Families`, the canonical family groupings and their listed member order (page order, not independently re-derived):

| Family | Description (quoted) | Members in listed order |
|--------|----------------------|--------------------------|
| Basic Strategies | "Introduction to Sudoku and fundamental techniques for solving puzzles systematically" | Introduction, Getting Started, Naked Pairs, Naked Triples, Naked Quads, Hidden Pairs, Hidden Triples, Hidden Quads, Pointing Pairs, Box/Line Intersection |
| 'Bent' Sets | "Strategies involving bent configurations of candidates within units" | Chute Remote Pairs, Y-Wing, W-Wing, XYZ-Wing, WXYZ-Wing, Almost Locked Pairs/Triples, Fireworks, Twinned XY-Chains |
| Chaining Strategies | "Themes involving bi-value and bi-location pairs and complex deduction chains" | X-Wing Family, Rectangle Elimination, Swordfish, Jellyfish, Simple Colouring, Multi-Colouring, Y-Wing Chains, XY-Chains, 3D Medusa, Remote Pairs, X-Cycles (Part 1), X-Cycles (Part 2), SK Loops, Grouped X-Cycles, Inference Chains, AIC with Groups, AIC with ALSs, AIC with URs, AIC with exotic links |
| Forcing Chains (subgroup) | UNKNOWN (no family description quoted on page) | Digit Forcing Chains, Nishio Forcing Chains, Cell Forcing Chains, Unit Forcing Chains |
| Exotic Strategies | "Strategies with distinctive logic including advanced pattern recognition" | Almost Locked Sets, Finned X-Wing, Finned Swordfish, Franken Swordfish, Pattern Overlay, Aligned Pair Exclusion, Empty Rectangles, Tridagons, Exocet, Double Exocet, Sue-de-Coq, Death Blossom, Bowman's Bingo, Multivalue X-Wing, Guardians |
| Uniqueness Strategies | UNKNOWN (no family description quoted on page) | Unique Rectangles, Extended Rectangles, Hidden Rectangles, Avoidable Rectangles, BUG+1, Gurth's Theorem |
| Jigsaw Strategies | "Specialized approaches leveraging irregular box shapes" | Double Pointing Sets, Double Line/Box Reduction, Law of Leftovers |
| Deprecated Strategies | UNKNOWN | Remote Pairs, Y-Wing Chain, Multivalue X-Wing, Multi-Colouring, Empty Rectangles, Guardians |

For each strategy in scope for this spine, per-page stated difficulty tier/position (where the individual strategy page itself states one — this is more granular than the family table above):

| Strategy | Family (from index) | Tier/position stated on its own page |
|----------|----------------------|----------------------------------------|
| Naked Singles | Basic Strategies | UNKNOWN numeric tier; page frames it as the foundational "eyeballing" strategy used in "every puzzle from the easiest to the hardest" |
| Hidden Singles | Basic Strategies (covered on Getting_Started, no standalone page) | Same as above — foundational, no numeric tier stated |
| Naked Pairs/Triples/Quads | Basic Strategies | UNKNOWN numeric tier on `/Naked_Candidates` itself; page calls Naked Pairs "the simplest such situation" beyond singles |
| Hidden Pairs/Triples/Quads | Basic Strategies | UNKNOWN numeric tier stated on `/Hidden_Candidates` |
| Intersection Removal (Pointing Pairs / Box-Line Reduction) | Basic Strategies | Page states both are "Listed under 'Basic Strategies'" |
| X-Wing | Chaining Strategies | "Classified under 'Tough Strategies'"; practice puzzles score 74–153 (pre-Oct-2025 scale, per fetched page) |
| Y-Wing | 'Bent' Sets | "Listed under 'Tough Strategies'"; exemplars graded 6.1–7.6 |
| XYZ-Wing | 'Bent' Sets | UNKNOWN explicit tier label in fetched excerpt; page frames it as an extension of Y-Wing |
| W-Wing | 'Bent' Sets | "Classified as a 'Tough' strategy"; page states it is "positioned after Y-Wings in difficulty progression" |
| Swordfish | Chaining Strategies | "Listed under 'Tough Strategies'"; exemplars score 6.3–7.9 |
| Simple Colouring | Chaining Strategies | "Classified as a 'Tough Strategy,' positioned between basic strategies and diabolical techniques like X-Cycles" |
| X-Cycles (Part 1) | Chaining Strategies | "Classified as 'Diabolical Strategies'"; page states "X-Cycles are strongly related to Simple Coloring" |
| Unique Rectangles | Uniqueness Strategies | UNKNOWN explicit numeric tier in fetched excerpt |
| BUG | Uniqueness Strategies | Puzzles requiring it "score between 3.2 and 5.3"; page states "every instance of BUG can be solved by an XY-Chain" |
| Chute Remote Pairs | 'Bent' Sets | Page states it "should follow X-Wing instruction and precedes Y-Wings"; exemplar puzzles range 6.5–8.5 |
| Rectangle Elimination | Chaining Strategies | Page's own text: "Family: Tough Strategies... Positioning: Between Y-Wing and Swordfish strategies"; also stated to replace the deprecated Empty Rectangles strategy |

## 4. Per-Strategy Entries

### Naked Singles
- **Family / URL**: Basic Strategies. https://www.sudokuwiki.org/Getting_Started
- **Grading tier / position**: Grading model gives point value `F` (the density factor itself; see Section 2). Framed as the foundational strategy on the page.
- **Preconditions (theory terms)**: A cell has only one remaining candidate because "every other number from 1 to 9 apart from [the target] is present in either the row, column or box."
- **Conceptual primitives consumed**: bivalue cell is not applicable here (this is the "univalue" endpoint) — closest listed primitive is none exactly; note as a new primitive: "single remaining candidate in a cell" (direct elimination exhaustion).
- **Prerequisite strategies**: UNKNOWN (none stated; it is presented first).
- **Canonical examples**: Page gives worked cell references without a full puzzle string: placing 8 at "Box 7, cell H1"; placing 4 at "Row A, cell A2"; placing 4 at cell J8; placing 5 at cell B1. No full grid/puzzle string given in the fetched excerpt. URL: https://www.sudokuwiki.org/Getting_Started
- **Elimination/placement type**: Placement (fills the cell with its single remaining candidate).

### Hidden Singles
- **Family / URL**: Basic Strategies. No standalone page — content lives on https://www.sudokuwiki.org/Getting_Started (`/Hidden_Singles` returns 404).
- **Grading tier / position**: Grading model gives point value `F × 2`.
- **Preconditions (theory terms)**: "other candidates are possible in those places but at least one candidate is unique to a particular row, column and box" — i.e., a candidate that appears only once across the remaining cells of some unit, even though the cell itself is not naked-single.
- **Conceptual primitives consumed**: group-candidate positions (transposed availability) — checking, per unit, where each candidate can still go, rather than per-cell.
- **Prerequisite strategies**: UNKNOWN (none stated).
- **Canonical examples**: Same worked example set as Naked Singles on the Getting_Started page (the page interleaves both types across its four worked examples); no separate hidden-singles-only example/grid string is isolated in the fetched excerpt. URL: https://www.sudokuwiki.org/Getting_Started
- **Elimination/placement type**: Placement.

### Naked Pairs / Naked Triples / Naked Quads
- **Family / URL**: Basic Strategies. https://www.sudokuwiki.org/Naked_Candidates
- **Grading tier / position**: Grading model gives Naked Pair = `5 × F`. Triples/Quads points: UNKNOWN. Page frames pairs as "the simplest such situation" beyond naked singles, triples/quads as "progressively rarer but more powerful."
- **Preconditions (theory terms)**: Naked Pair — "a set of two candidate numbers sited in two cells that belong to at least one unit in common." Naked Triple — "any group of three cells in the same unit that contain IN TOTAL three candidates," individual cells may hold 2 or 3 of them (formations {3/3/3}, {3/3/2}, {3/2/2}, {2/2/2}). Naked Quad — analogous with four cells / four candidates total.
- **Conceptual primitives consumed**: bivalue cell (for the pair case; triples/quads generalize to "cells restricted to a subset of N candidates").
- **Prerequisite strategies**: Builds on naked singles ("beyond naked singles" per page); no other prerequisite stated.
- **Canonical examples**:
  - Naked Pair, Figure 1: cells A2 and A3 both `[1,6]` in row A → removes other 1s/6s in row A, and (sharing a box) the 1 in C1. URL: https://www.sudokuwiki.org/Naked_Candidates
  - Naked Pair, Figure 2: cells H2 and J1 `[4,7]` in a shared box → removes other 7s in that box. Same URL.
  - Naked Triple example: row E, cells E4 `[5,8,9]`, E5 `[5,8]`, E6 `[5,9]` (formation {3/3/2}) → removes 5, 8, 9 from remaining row-E cells. Same URL.
  - Naked Quad example: box 1, cells A1, B1, B2, C1 collectively `[1,5,6,8]` → removes those four candidates elsewhere in box 1. Same URL.
- **Elimination/placement type**: Elimination (removes the naked-set candidates from all other cells sharing the unit(s)).

### Hidden Pairs / Hidden Triples / Hidden Quads
- **Family / URL**: Basic Strategies. https://www.sudokuwiki.org/Hidden_Candidates
- **Grading tier / position**: UNKNOWN numeric points (not among the six examples given on `/Grading_Puzzles`).
- **Preconditions (theory terms)**: Hidden Pair — two numbers appear only in the same two cells within a row/column/box, with other candidates also present in those cells ("clutter"). Hidden Triple/Quad — generalizes to three/four numbers confined to three/four cells in a unit "regardless of distribution." Page notes higher orders (hidden quins+) aren't considered because a unit only has 9 cells (a hidden quin would imply a complementary hidden quad).
- **Conceptual primitives consumed**: group-candidate positions (transposed availability) — the defining feature is scarcity of a candidate's positions within a unit, not scarcity of a cell's candidates.
- **Prerequisite strategies**: UNKNOWN (not stated).
- **Canonical examples**:
  - Hidden Pair: cells A8 and A9 hold candidates including 6 and 7; "6 and 7 have been found in the first two boxes" so they must occupy A8/A9 → "clear off all the alternatives" from A8/A9. URL: https://www.sudokuwiki.org/Hidden_Candidates
  - Hidden Triple: row A, cells A4 `[2,5,6]`, A7 `[2,6]`, A9 `[2,5]` — "these three cells are the last remaining cells in row A which can contain 2, 5 and 6" → other candidates removed from A4/A7/A9. Same URL.
- **Elimination/placement type**: Elimination (strips non-hidden-set candidates from the identified cells, effectively converting them into a naked set).

### Intersection Removal (Pointing Pairs / Box-Line Reduction)
- **Family / URL**: Basic Strategies. https://www.sudokuwiki.org/Intersection_Removal
- **Grading tier / position**: Page states both sub-strategies are "Listed under 'Basic Strategies.'" No numeric score given.
- **Preconditions (theory terms)**: Pointing Pairs/Triples — a candidate appears exactly twice or three times within a single box and those instances are aligned on one row or column, with no other instances of that candidate elsewhere in the box. Box/Line Reduction — a candidate, looked at from a row or column, appears only within a single box.
- **Conceptual primitives consumed**: group-candidate positions (transposed availability) — restricting a candidate's position within the intersection of a box and a line.
- **Prerequisite strategies**: UNKNOWN (not stated).
- **Canonical examples**:
  - Pointing Pair: "The 3s in B7 and B9 are alone in box 3 and they are aligned on the row. So looking along the row we can remove all the 3s in Box 1." URL: https://www.sudokuwiki.org/Intersection_Removal
  - Box/Line Reduction: "The only 2s left are in A4 and A5 ... we can eliminate 2 from B5, C4 and C5." Same URL.
- **Elimination/placement type**: Elimination (removes the candidate from the rest of the line, for Pointing; from the rest of the box, for Box/Line Reduction).

### X-Wing
- **Family / URL**: Chaining Strategies (per family index); page itself doesn't repeat the family name in the fetched excerpt. https://www.sudokuwiki.org/X_Wing_Strategy
- **Grading tier / position**: Grading model gives fixed 30 points (Section 2). Page's own text: "Classified under 'Tough Strategies'"; exemplar puzzles score 74–153 on what appears to be the pre-Oct-2025 scale (page text doesn't reconcile this with the new grading table, so treat these two numeric scales as reported separately, both sourced from sudokuwiki but possibly from different eras of the site).
- **Preconditions (theory terms)**: "only two possible cells for a value in each of two different rows, and these candidates lie also in the same columns" (or the row/column-swapped equivalent) — i.e. exactly-2-occurrences of a candidate in each of two units of one kind (conjugate-pair-like scarcity), aligned across two units of the other kind, forming a rectangle. Page states understanding of "locked pairs" (candidate confined to two cells in a unit) is assumed.
- **Conceptual primitives consumed**: strong link/conjugate pair; fish pattern n×n (n=2).
- **Prerequisite strategies**: Locked-pair/candidate-scarcity understanding is assumed per the page, but no explicit named prerequisite strategy is listed.
- **Canonical examples**:
  - Example 1: digit 7 forms a rectangle across cells "A, B, C, and D" — "any other 7s along the edge of our rectangle are redundant. We can remove the 7s marked in the green squares." (Exact cell labels for A/B/C/D not further specified in the fetched excerpt.) URL: https://www.sudokuwiki.org/X_Wing_Strategy
  - Example 2: cells E5/J8 and E8/J5 on candidate 2 → eliminates six other 2s across the specified rows. Same URL.
- **Elimination/placement type**: Elimination (removes the candidate from the two lines outside the four X-Wing cells).

### Y-Wing
- **Family / URL**: 'Bent' Sets. https://www.sudokuwiki.org/Y_Wing_Strategy
- **Grading tier / position**: "Listed under 'Tough Strategies'"; exemplars graded 6.1–7.6.
- **Preconditions (theory terms)**: Three bivalue cells: pivot cell with candidates {A,B}; two pincer cells with {A,C} and {B,C} respectively. The pivot must "see" (share row/column/box with) both pincers; the pincers need not see each other. Quote: "Three of the corners have two candidates AC, AB and BC. The cell marked AB is the key."
- **Conceptual primitives consumed**: bivalue cell (three of them, in the AB/AC/BC pattern).
- **Prerequisite strategies**: UNKNOWN explicitly, but Rectangle Elimination's own page states it is positioned "between Y-Wing and Swordfish," implying Y-Wing precedes it; Chute Remote Pairs' page states it "precedes Y-Wings," implying Chute Remote Pairs comes first.
- **Canonical examples**: Example 1: cell A1 `{7,1}` (pincer), A7 `{1,2}` (pincer), E1 `{7,2}` (pivot, per the AB/AC/BC labeling — page's exact pivot/pincer cell assignment is as quoted). Result: "2 in E7 can be removed" because both A7 and E1 "see" E7. URL: https://www.sudokuwiki.org/Y_Wing_Strategy
- **Elimination/placement type**: Elimination (removes the shared candidate C from any cell seeing both pincers).

### XYZ-Wing
- **Family / URL**: 'Bent' Sets. https://www.sudokuwiki.org/XYZ_Wing
- **Grading tier / position**: UNKNOWN explicit tier label in the fetched excerpt.
- **Preconditions (theory terms)**: Three cells with only 3 candidates between them total: a hinge/apex cell with all three candidates {X,Y,Z}; two pincer cells with two candidates each ({X,Z} and {Y,Z}). The hinge must see both pincers; the three cells fall within the same "chute" (row/column plus box configuration).
- **Conceptual primitives consumed**: bivalue cell (pincers) plus one trivalue cell (hinge); chute analysis (page explicitly invokes "chute" configuration for the three cells).
- **Prerequisite strategies**: Page states it "extends the Y-Wing pattern by adding an extra candidate to the hinge cell" — Y-Wing is the implied prerequisite.
- **Canonical examples**: Example 1: hinge F9, pincers D9 `{1,2}` and F1 `{1,4}`; eliminates digit 1 from F7. Reasoning quoted: "If D9 contains a 2 then F1 and F9 become a naked pair of 1/4 ... If F1's a 4 then D9 and F9 become a naked pair of 1/2." URL: https://www.sudokuwiki.org/XYZ_Wing
- **Elimination/placement type**: Elimination (removes the common candidate Z from any cell seeing all three cells of the pattern).

### W-Wing
- **Family / URL**: 'Bent' Sets. https://www.sudokuwiki.org/W_Wing_Strategy
- **Grading tier / position**: "Classified as a 'Tough' strategy"; page states it is "positioned after Y-Wings in difficulty progression."
- **Preconditions (theory terms)**: Two cells with identical bivalue candidates that cannot see each other directly, connected via a strong link on one of the two shared candidates. Quote: "two cells with 3 and 6 in them - cells A6 and F5. They cannot see each other. We cannot connect them through 3 but we can connect them through a strong link on 6." Page also describes three named variants: Double/Remote Pair Chain (four cells, alternating bivalue chain), Single W-Wing (two bivalue cells + one strong link), Split Double (two simultaneous W-Wings sharing endpoints, different routing).
- **Conceptual primitives consumed**: bivalue cell; strong link/conjugate pair; chain/graph traversal (for the multi-cell remote-pair-chain variant).
- **Prerequisite strategies**: "builds on understanding of strong and weak links"; explicitly "positioned after Y-Wings in difficulty progression."
- **Canonical examples**: Example 1: cells A6 and F5 both `{3,6}`; a strong link on candidate 6 proves one endpoint must be 3 → "3s are eliminated from D6 and E6." URL: https://www.sudokuwiki.org/W_Wing_Strategy
- **Elimination/placement type**: Elimination (removes the non-linked shared candidate from cells that can see both chain endpoints).

### Swordfish
- **Family / URL**: Chaining Strategies. https://www.sudokuwiki.org/Sword_Fish_Strategy
- **Grading tier / position**: "Listed under 'Tough Strategies'"; exemplars score 6.3–7.9.
- **Preconditions (theory terms)**: "a 3 by 3 nine-cell pattern where a candidate is found on three different rows (or three columns) and they line up in the opposite direction" — candidate confined to three rows/columns forming a 3×3 grid; variants include "Perfect 3-3-3" (all nine cells hold the candidate), "2-2-2" (candidate appears exactly twice in each of the three rows/columns), and combinations like "3-2-1."
- **Conceptual primitives consumed**: fish pattern n×n (n=3); group-candidate positions (transposed availability).
- **Prerequisite strategies**: X-Wing — page states "All Swordfishes will break down into X-Wings and because we know X-Wings work, so will the Swordfish," framing Swordfish as a generalization of the 2×2 X-Wing to 2×3/3×3.
- **Canonical examples**: No fully quoted worked cell-reference example was captured in the fetched excerpt beyond the general pattern description and variant names (Perfect 3-3-3, 2-2-2, 3-2-1). URL: https://www.sudokuwiki.org/Sword_Fish_Strategy — flagged `UNKNOWN (example detail not captured in fetch)`.
- **Elimination/placement type**: Elimination (candidate removed from all other positions on the three aligned rows/columns).

### Simple Colouring (Singles Chains)
- **Family / URL**: Chaining Strategies. https://www.sudokuwiki.org/Simple_Colouring
- **Grading tier / position**: "Classified as a 'Tough Strategy,' positioned between basic strategies and diabolical techniques like X-Cycles."
- **Preconditions (theory terms)**: Operates on one candidate number at a time (contrasted with multi-colouring/3D Medusa). Requires bi-location strong links: "Links form where exactly two instances of a candidate exist within any single unit ... Where three or more candidates exist, no links are possible within that unit." Chain construction alternates two colors along links; "A candidate can either be ON or OFF."
- **Conceptual primitives consumed**: strong link/conjugate pair; chain/graph traversal (two-coloring of the conjugate-pair graph for a single digit).
- **Prerequisite strategies**: UNKNOWN explicit prerequisite stated, though it is textually distinguished from Multi-Colouring/3D Medusa as the single-digit case.
- **Named elimination rules** (as quoted):
  - Rule 2 – "Twice in a Unit": "If any unit has the same colour twice ALL those candidates which share that colour must be OFF. The alternative colour will be ON and the solution for that cell."
  - Rule 4 – "Two Colours Elsewhere": "If you can spot a candidate X that can see an X of both colours - then it must be removed."
  - Rule 7 – "One Colour Empties a Unit": if one color, if ON, would eliminate all remaining candidates in some unit, that color cannot be ON.
  - Page statistic: from Rudd's top 50,000 puzzles, 2,709 (2.65%) contain Simple Colouring; Rule 4 accounts for 90% of eliminations.
- **Canonical examples**: UNKNOWN — no specific worked grid/cell-reference example was captured in the fetched excerpt (only the rule statements and statistics). URL: https://www.sudokuwiki.org/Simple_Colouring
- **Elimination/placement type**: Both — Rule 2 leads to a placement (opposite color becomes the solution); Rules 4 and 7 are eliminations.

### X-Cycles (Part 1)
- **Family / URL**: Chaining Strategies. https://www.sudokuwiki.org/X_Cycles (Part 2 at `/X_Cycles_Part_2` not fetched)
- **Grading tier / position**: "Classified as 'Diabolical Strategies'"; page states "X-Cycles are strongly related to Simple Coloring" and represents "advancement from basic strategies."
- **Preconditions (theory terms)**: Strong link — "if not A, then B" (`!A ⇒ B`), occurring when exactly two candidates exist in a unit. Weak link — "if A, then not B" (`A ⇒ !B`), occurring when three or more candidates exist in a unit. A chain is "a series of links hopping from one candidate to another" for a single digit, alternating strong/weak around a closed loop. "Nice Loops that alternate all the way round are said to be 'continuous', and they must have an even number of nodes."
- **Conceptual primitives consumed**: strong link/conjugate pair; chain/graph traversal (cycle-specific, single digit).
- **Prerequisite strategies**: Requires understanding of strong/weak link concepts per the page; explicitly related to (and framed as building on) Simple Colouring.
- **Named elimination rules**:
  - Nice Loops Rule 1 (Continuous Loops): "we are looking to eliminate on the units that can be seen by two or more cells that belong to the loop" — off-chain candidates removed at weak-link units.
  - Nice Loops Rule 2 (Discontinuous Loops): described as documented in Part 2 (not fetched) — `UNKNOWN (not fetched)` beyond that pointer.
- **Canonical example**: Figure 4 "real-life puzzle," X-Cycle on digit 8, chain notation `-8[A1]+8[A6]-8[C4]+8[H4]-8[H2]+8[J1]-8[A1]`. Stated eliminations: "Off-chain 8 taken off B6," "Off-chain 8 taken off C5," "Off-chain 8 taken off H7." URL: https://www.sudokuwiki.org/X_Cycles
- **Elimination/placement type**: Elimination (continuous-loop rule); Rule 2 (discontinuous loops, per Part 2) may include placements — detail `UNKNOWN (not fetched)`.

### Unique Rectangles
- **Family / URL**: Uniqueness Strategies. https://www.sudokuwiki.org/Unique_Rectangles
- **Grading tier / position**: UNKNOWN explicit numeric tier in the fetched excerpt.
- **Preconditions (theory terms)**: Uniqueness assumption — "published Sudokus have only one solution." Rectangle formation: four cells spanning exactly two rows, two columns, and two boxes, all four holding the same candidate pair (e.g., 2/9). "Deadly Pattern": if the pair spans only two boxes (not four), two symmetric solutions would exist, contradicting uniqueness — this is disallowed, licensing eliminations. If the pair spans four boxes instead, the pattern is valid and doesn't imply multiple solutions (no deduction available).
- **Conceptual primitives consumed**: uniqueness assumption; bivalue cell (all four rectangle cells restricted, at minimum, to the base pair).
- **Prerequisite strategies**: UNKNOWN explicitly stated; page notes "Type 5 is searched after Type 1 but before other variants" and "Type 4 variants should generally be explored only after other possible reductions are exhausted, as they 'destroy' the rectangle structure" — an internal type-ordering rather than a named external prerequisite.
- **Types and elimination rules** (as quoted/summarized):
  - Type 1: three cells hold only the base pair (floor); the fourth (corner) holds the pair plus extra candidates → both base candidates removed from the corner cell.
  - Type 2: floor cells in one box, roof cells in another, roof cells share one extra candidate → "7 must appear in either A5 or A6 (the roof squares). Therefore, it can be removed from all other cells in the units (row, column and box) that contain both of the roof cells."
  - Type 2B: floor/roof split one-per-box → eliminations limited to shared row/column, not box.
  - Type 2C: extra candidate on diagonally opposite corners → eliminate from cells seeing both diagonal cells.
  - Type 3: roof cells hold the base pair plus two extra candidates forming a "pseudo-cell"; combined with a bivalue cell holding those same extras, forms a locked set → eliminates those extras elsewhere in the shared unit.
  - Type 3b: floor cells in different boxes, roof cells share a box; locked set formed with a bivalue cell in that box or aligned row/column.
  - Type 4: roof cells hold the base pair; one base candidate is a conjugate pair within the roof cells' shared unit → "since the roof squares are the only squares that can contain a 6 ... neither of the squares can contain a 7, since this would create the deadly pattern" → the other base candidate removed from both roof cells.
  - Type 4B: floor cells in different boxes; conjugate-pair search restricted to shared row/column only.
  - Type 5: opposite corners hold only the base pair; one candidate has strong links to adjacent corners → "If 2 (the weakly linked candidate in the pair) was ON in either E6 or F1 it would force 8 to be in the other corners. This is not allowed so 2 can be removed from the Naked Pair."
- **Canonical examples**: Type 1 worked description: three cells hold only 2/9, fourth holds 2/9 plus 1/5 → remove 2 and 9 from that cell (illustrative, no full grid string captured). Type 4 example candidates 6/7 with roof cells (exact cell labels not captured in fetched excerpt beyond the quoted reasoning). URL: https://www.sudokuwiki.org/Unique_Rectangles — flagged `UNKNOWN (exact cell coordinates for several type examples not captured in fetch)`.
- **Elimination/placement type**: Elimination (all types remove candidates; page's fetched excerpt gives no explicit placement-only UR case).

### BUG (Bivalue Universal Grave)
- **Family / URL**: Uniqueness Strategies (page title-indexed as "BUG+1" in the family list). https://www.sudokuwiki.org/BUG
- **Grading tier / position**: Puzzles requiring it "score between 3.2 and 5.3."
- **Preconditions (theory terms)**: "all remaining cells contain just two candidates" except for exactly one cell with three candidates. Core principle quoted: "any Sudoku where all remaining cells contain just two candidates is fatally flawed. There would have been a last remaining cell with three candidates" — i.e., a uniqueness-assumption argument (an all-bivalue grid has two symmetric solutions and thus cannot be the unique-solution end state of a valid puzzle).
- **Conceptual primitives consumed**: uniqueness assumption; bivalue cell (as the near-universal grid state); group-candidate positions (counting a candidate's occurrences in the trivalue cell's row/column/box).
- **Prerequisite strategies**: UNKNOWN explicit prerequisite; page states "every instance of BUG can be solved by an XY-Chain," positioning it as equivalent in power to (but distinct in method from) XY-Chains.
- **Canonical example**: Puzzle example with cell F8 holding candidates {3,4,6}. Candidate 3 appears three times in row F, column 8, and its box → 3 is the solution for F8. Reasoning: "If candidate 4 were placed instead, two naked 6s would result in the row. If candidate 6 were placed, two naked 4s would appear." URL: https://www.sudokuwiki.org/BUG
- **Elimination/placement type**: Placement (the trivalue cell is resolved to the one candidate satisfying the count rule; the other two candidates are, implicitly, eliminated from that cell).

### Chute Remote Pairs
- **Family / URL**: 'Bent' Sets. https://www.sudokuwiki.org/Chute_Remote_Pairs
- **Grading tier / position**: Page states it "should follow X-Wing instruction and precedes Y-Wings." Exemplar puzzles range 6.5–8.5 in difficulty score. Testing note quoted: "I've found some 620 examples in 2000 random diabolicals and I've gained an overall speed increase of about 2%."
- **Preconditions (theory terms)**: Two bivalue cells with identical candidate pairs, located in the same "chute" (a horizontal or vertical set of three boxes), that do not see each other (ruling out plain Naked Pair). Quote: "We're looking for two bi-value cells with the same candidates in the same Chute, which is a horizontal or vertical set of the three boxes."
- **Conceptual primitives consumed**: bivalue cell; chute analysis (explicitly named on the page).
- **Prerequisite strategies**: Page states it should be taught/applied "after X-Wing" and "precedes Y-Wings" — an ordering relative to those two strategies, not a strict logical prerequisite.
- **Canonical example**: Green cells A8 and C1, both `{4,7}`, in the same chute; yellow cells B4/B5/B6 (the unused third box of the chute) contain only one of the two candidates (e.g., only 4) → that candidate (4) is removed from all cells seen by both A8 and C1. URL: https://www.sudokuwiki.org/Chute_Remote_Pairs
- **Elimination/placement type**: Elimination.

### Rectangle Elimination
- **Family / URL**: Chaining Strategies (per family index; the page's own text also labels it "Family: Tough Strategies," which is a difficulty-tier label rather than the site's topical family). https://www.sudokuwiki.org/Rectangle_Elimination
- **Grading tier / position**: Page: "a 'tough' solving strategy that replaces Empty Rectangles"; "Positioning: Between Y-Wing and Swordfish strategies." Prevalence noted: "approximately 35,826 instances across 23,885 puzzles in testing."
- **Preconditions (theory terms)**: Operates on a single candidate number. Requires a "hinge cell" with a strong link to one remaining candidate position in its row or column; a second "wing" cell in the opposite orientation (different row/column) within a different box, weakly linked (more than two instances of the candidate in that unit).
- **Conceptual primitives consumed**: strong link/conjugate pair (hinge); group-candidate positions (transposed availability, for the weak-link wing); chute/box-corner reasoning ("fourth corner box" argument).
- **Prerequisite strategies**: UNKNOWN explicitly named, though its stated position "between Y-Wing and Swordfish" implies those two bracket it in the site's teaching order. Explicitly stated to replace/supersede the deprecated Empty Rectangles strategy.
- **Canonical example**: Candidate 9; hinge cell G2, first wing G6 (strong link in row), second wing A2 (weak link in column), "fourth corner" = box 2. Reasoning quoted: "Consider the weakly-linked A2. If it's ON, then the other wing cell G6 must also be ON. However, this would eliminate ALL the 9s in the 'fourth corner box' (box 2, which is the fourth corner of the rectangle). So A2 cannot be ON." Result: 9 removed from A2. URL: https://www.sudokuwiki.org/Rectangle_Elimination
- **Elimination/placement type**: Elimination.

## Notes on Gaps

- No standalone "Hidden Singles" page exists on sudokuwiki.org; its content is folded into `/Getting_Started` alongside Naked Singles. This is a confirmed structural fact (404 + search cross-check), not a fetch failure to retry further.
- `/X_Cycles_Part_2` (discontinuous-loop Nice Loops Rule 2 detail) was not fetched; flagged `UNKNOWN (not fetched)` in the X-Cycles entry.
- Several strategies (Naked/Hidden Triples/Quads, Intersection Removal, Y-Wing, XYZ-Wing, Simple Colouring, X-Cycles, Unique Rectangles, Chute Remote Pairs, Rectangle Elimination) have no point-value entry in the `/Grading_Puzzles` table as fetched — that page's excerpt only enumerated six example rows, not a complete strategy→points mapping.
- Some worked examples (Swordfish, Simple Colouring) did not yield a fully quoted cell-by-cell puzzle example in the fetched excerpt; these are flagged inline rather than filled from training knowledge.
