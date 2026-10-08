# LockedSets v3 — sources

Working list of literature for the matching-based (Régin) implementation.
Canonical record: Zotero collection `LockedSets v3` (key `8CBEMBNR`), tags `locked-sets` + tier. Read via `zotero-cli`.
Status: `verified` = URL fetched and content checked this session; `doi-resolved` = DOI resolved through CrossRef on import; `unverified` = citation from memory.

## Core — read in this order

### 1. van Hoeve 2001 — The alldifferent Constraint: A Survey — `PXLSRZ5C`
- W.J. van Hoeve, arXiv:cs/0105015, https://arxiv.org/abs/cs/0105015 (`verified`, open access, PDF attached in Zotero)
- Why: clearest exposition; introduces every concept in one place with consistent notation.
- Need from it:
  - §2 Preliminaries, Theorem 1 (Hall): the condition behind "locked set"; a naked k-set is a tight Hall set.
  - §6 Hyper-arc consistency: the consistency level `find_locked_set_v1` computes.
  - §6.1 Connections with matching theory: edge in some maximum matching ⇔ candidate survives.
  - §6.2 Algorithm: matching + oriented graph + SCC, the v3 skeleton.
- Note: the fetch summariser reported wrong section numbers (1.2/1.3/1.7); the ones above are from the PDF text.

### 2. Régin 1994 — A Filtering Algorithm for Constraints of Difference in CSPs — `ZIXQG9FN`
- J.-C. Régin, AAAI-94, pp. 362–367, https://cdn.aaai.org/AAAI/1994/AAAI94-055.pdf (`verified`; "all rights reserved", keep local only)
- The AAAI scan has no text layer: read it with `zotero-cli read ZIXQG9FN --format image`.
- Why: the original algorithm; the one to cite in code.
- Need from it:
  - Theorem 1 (p. 363): correctness — GAC ⇔ every edge lies in a matching covering X.
  - Property 1 (Berge) + Proposition 1 (p. 364): oriented graph `G_O`, `Succ`; even alternating cycle ⇔ directed cycle.
  - Algorithm 2 `RemoveEdgesFromG` (p. 365): the removal procedure. Step 2 (BFS from free vertices) vanishes for us: a group has |cells| = |values|, the matching is perfect, no free vertex exists. Only step 3 (SCC) remains. Needs a written proof sketch in our notes.
  - Algorithm 3 `Diff-Propagation` (p. 365): incremental repair after deletions; relevant only if a persistent variant is pursued later.
  - "vital" edge = candidate in every matching = naked single (observation, verify).

### 3. Gent, Miguel, Nightingale 2008 — Generalised arc consistency for the AllDifferent constraint: An empirical survey — `MT3I5LYU`
- I.P. Gent, I. Miguel, P. Nightingale, Artificial Intelligence 172(18), 2008, doi:10.1016/j.artint.2008.10.006 (`doi-resolved`; PDF attached via GUI as `NCST75TX`, born-digital: `get fulltext`, `read --find` and Zotero's index all work)
- Why: compares implementation choices (matching algorithm, SCC vs. alternatives, incremental SCC, bit-level domains) with measurements. Directly informs the "internal crossover" question (bitset closure at N=9 vs. Tarjan at N=25) and whether incremental is worth it.
- Need from it: the variants table and which ones won on small domains; any bitset-specific tricks.

## Sudoku-specific

### 4. Simonis 2005 — Sudoku as a Constraint Problem — `X7HIV3TG`
- H. Simonis, CP 2005 Workshop on Modeling and Reformulating CSPs (PDF attached via GUI as `JUAJ8SWF`, born-digital: `get fulltext`, `read --find` and Zotero's index all work)
- Why: maps human techniques (naked/hidden subsets, X-Wing, …) onto propagation levels (forward checking, bound, hyper-arc). Tells us which of our strategies alldifferent-GAC subsumes and which it does not (nothing crossing groups).
- Need from it: the technique ↔ consistency-level table; the "GAC on all 27 alldifferents" solve-rate numbers.

## Foundations — cite only, no need to read

### 5. Hall 1935 — On Representatives of Subsets — `KV265JY2`
- P. Hall, J. London Math. Soc. s1-10(1):26–30, doi:10.1112/jlms/s1-10.37.26 (`doi-resolved`, no open-access PDF)
- Hall's theorem. Covered adequately by source 1.

### 6. Tarjan 1972 — Depth-First Search and Linear Graph Algorithms — `Q5USWHAB`
- R. Tarjan, SIAM J. Comput. 1(2):146–160, doi:10.1137/0201010 (`doi-resolved`, no open-access PDF)
- SCC algorithm. Standard; any textbook version suffices. Likely not even needed at N ≤ 25 (bitset transitive closure may win).

### 7. Hopcroft, Karp 1973 — An n^5/2 Algorithm for Maximum Matchings in Bipartite Graphs — `RKPBBFD6`
- J.E. Hopcroft, R.M. Karp, SIAM J. Comput. 2(4):225–231, doi:10.1137/0202019 (`doi-resolved`, no open-access PDF)
- Asymptotically best matching. Overkill at N ≤ 25; the plain augmenting-path algorithm (Kuhn) suffices. Listed so the choice is documented.

### 8. Berge 1970 — Graphes et Hypergraphes — `BIXFG4KF`
- C. Berge, Dunod, Paris. Property 1 in Régin. Out of print, French; take it via Régin.

## Secondary — worked implementations

### 9. opensourc.es — Sudoku / constraint-solver series (Julia) — `W2Q3M7IV`, `VZ4CWQGD`
- https://opensourc.es/blog/sudoku/ and https://opensourc.es/blog/constraint-solver-alldifferent (`verified`)
- Matching + SCC filtering for sudoku, from scratch, non-incremental, benchmarked. Useful as a sanity reference for behaviour, not for performance choices.

## Considered and excluded
- Puget 1998 (bound consistency), López-Ortiz et al. 2003 (fast bound consistency): work on interval domains. Sudoku values are unordered symbols; bound/range consistency is meaningless here.
- Mohr & Masini 1988 GAC4: tuple enumeration, what v1 effectively does. Superseded.

## Open questions for the reading
- Proof that step 2 of Régin's Algorithm 2 is empty for a group (perfect matching ⇒ no free vertices).
- Whether Gent et al. measured anything at domain sizes as small as 9–25.
- Whether an SCC-based "reason" (the sink SCC as the naked set) matches what Simonis calls the naked subset — needed for `Reason::Candidates`.
