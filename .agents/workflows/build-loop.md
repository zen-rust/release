# Build loop — per-cycle process

Run this cycle for each task until `status/progress.md` shows all critical-path phases done and green.

## Cycle

1. **Pick** — open `status/progress.md`, take the top unblocked task in the current phase. If blocked on an open question, either resolve it from `decisions/decisions.md`/`AGENTS.md` or record the blocker and move to the next unblocked task.
2. **Plan** — restate the task and the acceptance criterion from `plans/build-plan.md`. Note the target files.
3. **Implement** — smallest coherent change that advances the task. Match surrounding code style.
4. **Gate** — run `skills/quality-gate.md`. Must be green before the task is "done". If `config.rs` changed, regenerate `.schema/latest.json`.
5. **Record** — check the task off in `progress.md`, add a dated line to its Progress log, update Gate status. Record any new decision in `decisions/decisions.md`.
6. **Repeat.**

## Rules

- Leave the tree compiling and gate-green at the end of every cycle. Never mark a task done on a red gate.
- Don't regress what already works (dependency ordering, index waits, idempotency/resumability) — see the "Already works" table in the feature review.
- Preserve the intentional upstream references (see decisions log) — don't scrub release-plz issue/PR links or inherited changelogs.
- Keep `AGENTS.md` as the spec; capture *decisions* in the decisions log, *state* in progress.md.
- If a change is outward-facing/irreversible (publishing, pushing, posting), stop and surface it — don't do it autonomously.
- Prefer small, reviewable commits per task (only if/when committing is requested).

## Overnight autonomy

- Work the critical path first: Phase 1 → 2 → 3.
- If a phase is genuinely blocked on a user decision, skip to the next unblocked task and clearly flag the blocker at the top of `progress.md` so it's the first thing seen in the morning.
- Do not spin up multi-agent workflows or publish anything without explicit user opt-in.
