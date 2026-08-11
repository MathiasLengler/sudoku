# Phase 0 — Manifest & Review Guide

Research run: "Analyse Copilot Slop → Strategy Roadmap" (2026-07-04).
Plan: `~/.claude/plans/this-is-a-planing-research-stateful-fiddle.md`

## Diff base

All `copilot/*` branches fork from the same commit on `dev`:

- Merge-base: `b87a1b3a` — "Merge remote-tracking branch 'origin/copilot/add-final-grid-metric-notification' into dev"
- Canonical diff invocation used by all Phase-2 agents: `git diff b87a1b3a...origin/copilot/<branch>`

## Branch → agent assignment

| Phase-2 report | Branches |
|---|---|
| `2-wings.md` | `origin/copilot/implement-y-wing-strategy`, `origin/copilot/implement-xyz-wing-strategy`, `origin/copilot/add-w-wing-strategy` |
| `2-fish.md` | `origin/copilot/add-swordfish-strategy` (vs existing `x_wing.rs`) |
| `2-chains.md` | `origin/copilot/add-simple-colouring-strategy`, `origin/copilot/implement-x-cycles-strategy` |
| `2-unique-rectangles.md` | `origin/copilot/implement-unique-rectangles-strategy`, `origin/copilot/add-rectangle-elimination-strategy` |
| `2-misc.md` | `origin/copilot/implement-new-bug-strategy`, `origin/copilot/add-chute-remote-pairs-strategy`, `origin/copilot/add-locked-sets-reasoning` |
| `2-infra-group-availability.md` | `origin/copilot/add-persistent-group-availability` |
| `2-sat-pruning.md` | `origin/copilot/optimize-sat-pruning` |

Out of scope: `evaluate-feature-gaps` (pure product/UX docs), all non-solver branches (UI, router, rendering, generator-config, …).

## File map / write ownership

Single writer per file; reads only across phase barriers; no cleanup — everything here is a review artifact.

| File | Writer | Phase |
|---|---|---|
| `0-manifest.md`, `0-branch-survey.md`, `0-codebase-baseline.md` | main session | 0 |
| `1-theory-spine.md` | Sonnet agent (WebFetch sudokuwiki.org) | 1 |
| `2-*.md` (7 files, see table above) | one Sonnet agent each | 2 |
| `3a-hierarchy-infra.md` | Opus agent (human-review gate after this) | 3 |
| `3b-sat-fit-roadmap.md` | Opus agent | 3 |
| `../.todo/strategy-roadmap.md` | main session | 4 |

## Review order suggestion

1. `1-theory-spine.md` — is the sudokuwiki grounding sound? (`UNKNOWN` markers = page didn't state it)
2. `2-*.md` — per-branch extraction; crosscheck claims via `git show <branch>:<file>` before deleting branches
3. `3a-hierarchy-infra.md` — hierarchy + infra proposals (gate: reviewed before roadmap was generated)
4. `3b-sat-fit-roadmap.md` → `.todo/strategy-roadmap.md`
