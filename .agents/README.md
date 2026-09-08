# `.agents/` — build coordination workspace

Working/coordination layer for building **zen-release** to the spec in the repo-root `AGENTS.md`. `AGENTS.md` is the *spec* (what to build and why); this directory is the *execution layer* (what's done, what's next, how we work).

## Layout

| Path | Purpose |
|---|---|
| `status/progress.md` | **Single source of truth** for state. Updated every work cycle. Read this first. |
| `plans/build-plan.md` | The phased engineering plan: tasks, acceptance criteria, file targets. |
| `decisions/decisions.md` | Decision log — locked naming/config/licensing calls. Don't re-litigate. |
| `workflows/build-loop.md` | The per-cycle process: pick task → implement → gate → record. |
| `skills/quality-gate.md` | Exact commands for the fmt + clippy + test gate ("done" definition). |

## The goal

Turn the branded engine into zen-release per `AGENTS.md`: config-driven multi-registry publishing (private-primary + optional rate-safe crates.io mirror), self-contained dep rewriting, tag-based versioning, opt-in whole-repo release PR. Critical path: **Phase 1 → 2 → 3** (registry config → self-containment → mirror).

## How to work here

1. Open `status/progress.md`; take the next unblocked task.
2. Follow `workflows/build-loop.md`.
3. Every change must pass the gate in `skills/quality-gate.md` before it's marked done.
4. Record progress + any new decision back into the tracker/log. Leave the tree green.
