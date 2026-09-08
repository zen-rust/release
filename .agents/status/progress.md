# zen-release — build progress

**Target:** feature-complete per `AGENTS.md`, critical path (Phases 1–3) working, by morning.
**Last updated:** 2026-09-07 (evening) — kickoff.

## Gate status

- `cargo check --workspace --all-targets` (stable 1.98.1): **GREEN** ✅
- fmt + clippy `--all-features -D warnings`: **GREEN** ✅ (after Phase 1 config)
- Tests: config unit tests **GREEN**; full test suite (incl. `docker-tests`) not yet run this session
- Toolchain: stable 1.98.1 (deps require ≥1.96); nightly 1.96 also available.

## Done (this session)

- [x] Full `release-plz` → `zen-release` rebrand (crates, binary, config, CI, docs); upstream links & changelogs preserved.
- [x] `crates/` → `packages/`; dirs `release` / `release_core`; crate names `zen_release` / `zen_release_core`; binary `zen-release` (via `[[bin]]`).
- [x] Env vars → `ZEN_RELEASE_*`; config file `zen-release.toml`.
- [x] License: MIT © Jetstream Labs (`LICENSE.md`); release-plz MIT retained in `licenses/RELEASE-PLZ.md`; `Cargo.toml license = "MIT"`.
- [x] All first-party crate versions reset to `0.0.1` + root `workspace.package.version = "0.0.1"`.
- [x] GH Actions workflows moved out of `.github/` → repo-root `workflows/` (none run). Docker → GHCR.
- [x] Feature review complete → see `plans/build-plan.md`.

## Phases (see plans/build-plan.md for task detail)

| # | Phase | Priority | Status |
|---|---|---|---|
| 1 | Config model: `[project]` + `[[registry]]` array | critical-path | ◐ in progress |
| 2 | Self-containment: rewrite internal deps to `registry = "…"` | critical-path | ☐ not started |
| 3 | crates.io mirror + rate-safe batching + 429 retry | critical-path | ☐ not started |
| 4 | Bump defaults (Jetstream ruleset) + tag-baseline default | important | ☐ not started |
| 5 | Whole-repo release PR + opt-in flag (default false) | secondary | ☐ not started |
| 6 | Monorepo subtree config (`packages_dir`) | low | ☐ not started |

## Phase 1 sub-tasks

- [x] `[project]` (`packages_dir`, `lockstep`, `tag_format`) + `[[registry]]` (`name`, `kind`, `index`, `token`, `self_contained`, `strip_alt_registry`, `batch_size`, `batch_gap_secs`, `retry_on_429`) structs in `config.rs`; `RegistryKind` enum (kellnr|sparse|crates-io). Serde + schemars.
- [x] `validate_registries()` (non-empty + unique names) wired into config load (`config_path.rs`).
- [x] `.schema/latest.json` regenerated. Gate green: fmt + clippy `-D warnings` + config unit tests pass.
- [ ] Thread registries into the publish flow (`ReleaseRequest` at `release.rs:37` is single `registry`/`token`) → primary + mirror list. (Overlaps Phase 2/3 wiring.)
- [ ] Add a round-trip parse test for a primary + mirror `zen-release.toml`.

## Current focus

→ **Phase 1** config structs + validation + schema **done & gate-green**. Next: add round-trip test, then Phase 2 (self-containment). Proceeding with the proposed `[[registry]]` shape — user started the build loop; shape recorded in `decisions/decisions.md`.

## Open questions / blockers

- [x] ~~Confirm `[[registry]]` config keys~~ — resolved: proceeding with the AGENTS.md/proposed shape (user started the build loop). Can still be adjusted.
- [ ] crates.io mirror placement: inline vs server-side webhook (AGENTS.md open decision #2) — MVP assumes inline.

## Progress log

- 2026-09-07 — Rebrand + restructure + version reset landed, gate green. Feature review done; `.agents/` scaffold created.
- 2026-09-07 — Build loop started (cron every 5m). **Phase 1 config model landed**: `[project]` + `[[registry]]` structs, `RegistryKind`, load-time validation, schema regenerated. Gate green (fmt + clippy -D warnings + config tests). Next: round-trip test + thread into publish flow, then Phase 2.
