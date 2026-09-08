# zen-release — phased build plan

Derived from the feature review (current engine vs. `AGENTS.md`). File:line refs point at the current code to change. Critical path: **1 → 2 → 3**.

Legend: acceptance = what must be true to mark the phase done (all under the quality gate).

---

## Phase 1 — Config model: `[project]` + `[[registry]]`

**Why:** Today registry is a single `--registry` CLI arg (`packages/release/src/args/update.rs:64`); the config file has no registry/mirror concept (`config.rs:22`). Everything in Phases 2–3 needs a config-driven registry list.

**Tasks**
- [ ] Add `[project]` section: `packages_dir`, `lockstep`, `tag_format` (map to existing tag templating).
- [ ] Add `[[registry]]` array struct: `name`, `kind` (kellnr | crates-io | sparse), `index`, `token` (env ref), `self_contained: bool`, `strip_alt_registry: bool`, `batch_size`, `batch_gap_secs`, `retry_on_429`. Derive `serde` + `schemars`.
- [ ] Distinguish exactly one primary vs. optional mirror(s) (validate at load).
- [ ] Thread registries into `ReleaseRequest` (`release.rs:37` currently a single `registry`/`token`).
- [ ] Regenerate `.schema/latest.json` (`generate-schema` subcommand; test at `generate_schema.rs:34` enforces freshness).

**Acceptance:** a `zen-release.toml` with a primary + mirror parses, round-trips through schema, and is available to the publish flow. No behavior change yet if only a primary is defined.

---

## Phase 2 — Self-containment: rewrite internal deps to `registry = "…"`

**Why:** `packages/cargo_utils/src/local_manifest.rs` rewrites only `version` (`:181,:210`); nothing writes a dependency's `registry` key. Needed so private-registry crates resolve internally before crates.io catches up.

**Tasks**
- [ ] Add a manifest transform: for each internal (workspace) dependency, set `registry = "<primary>"` when `self_contained = true`.
- [ ] Apply in the temp-copy publish path (before `cargo publish` to primary), never in the dev's working tree.
- [ ] Ensure it composes with the existing version rewrite.

**Acceptance:** publishing to the primary registry produces crates whose internal deps carry `registry = "<primary>"`; a fresh consumer resolving only against the primary index builds.

---

## Phase 3 — crates.io mirror + rate-safe batching

**Why:** *Absent today.* Publish is a plain sequential loop (`release.rs:626`); no batching, no 429/backoff, no mirror. This is the headline feature.

**Tasks**
- [ ] Mirror step that runs after the primary publish for registries with `kind = crates-io`.
- [ ] **Strip** `registry = "…"` back to plain deps for the crates.io manifest form (inverse of Phase 2).
- [ ] Publish in dependency-ordered **batches** (`batch_size`) with `batch_gap_secs` between batches.
- [ ] **429 handling:** detect rate-limit from `cargo publish` stderr; defer/retry with backoff (`retry_on_429`).
- [ ] Idempotent: reuse `is_published` (`release.rs:688`) to skip already-published; resumable across partial runs.
- [ ] Reuse `release_order` + `wait_until_published` for ordering/index waits.

**Acceptance:** a lockstep release of N crates (N > batch_size) publishes to the mirror across batches without tripping the crates.io burst limit; re-running completes any skipped crates; a mid-run 429 defers rather than fails.

**Wiring recipe (primitives all exist and are unit-tested — this is the remaining integration, needs Docker to verify):**
1. Thread mirror registries into `ReleaseRequest` (e.g. `mirror_registries: Vec<MirrorConfig>` with name/strip_alt_registry/batch_size/batch_gap_secs/retry_on_429). Populate in `config.fill_release_config` from `self.registry.get(1..)` where `kind == crates-io`.
2. In `release_packages`, after the primary publish loop completes, for each mirror registry run a mirror pass:
   a. Order packages via the existing `project.publishable_packages()` (already release-ordered).
   b. Strip internal-dep registry markers with `Manifest::strip_dependencies_registry` in the checkout, guarded by a `ManifestBackup` (same pattern as `self_contained.rs`) so the tree is restored after.
   c. `into_batches(packages, batch_size)`; between batches `tokio::time::sleep(batch_gap_secs)`.
   d. For each package: skip if `is_published` (idempotent); else `run_cargo_publish` to `--registry <mirror>` (or crates.io default). On failure, if `is_rate_limited(stderr)` and `retry_on_429`, defer (re-queue / backoff) instead of bailing; reuse `wait_until_published` for index waits.
3. crates.io token: reuse `find_registry_token` / trusted-publishing path already in `release_package`.
4. Tests: unit-test the batch/skip/defer control flow by injecting a fake publish closure (decouple from real cargo); integration-test the real publish under the docker suite.
Note: because self-contained (primary) and stripped (mirror) forms are both needed in one run, factor the "backup → transform → publish → restore" into a shared helper so primary and mirror passes reuse it.

---

## Phase 4 — Bump defaults (Jetstream) + tag-baseline default

**Why:** Engine supports it but defaults/plumbing are off. Only 3 of 5 bump knobs are wired (`update_config.rs:120`); tag baseline is per-package `git_only`, not the default.

**Tasks**
- [ ] Plumb `breaking_always_increment_major` and `no_increment_regex` through `UpdateConfig::version_updater()`.
- [ ] Ship Jetstream defaults: `custom_major_increment_regex = "^breaking"`; `no_increment_regex` covering `wip|chore` (+ scopes `style|test`); refactor/docs → patch (already default).
- [ ] Make tag-based baseline the default (global), so no registry read is required to version.

**Acceptance:** on a repo with `breaking:`/`feat:`/`fix:`/`wip:` commits and no registry access, the computed bump matches the Jetstream table in `AGENTS.md`.

---

## Phase 5 — Whole-repo release PR + opt-in flag (default false)

**Why:** Release-on-push already works (`release_always=true`, `release.rs:782`). Gaps: PR is command-selected not a config bool; PR scope is versions+changelogs+lock only.

**Tasks**
- [ ] Config flag `release_pr` (or `[pr] enabled`), **default false**.
- [ ] Widen PR scope to the whole repository, not just the crates subtree (commit step already `git add`s the temp copy — `update/mod.rs:504`).
- [ ] Keep the shared forge layer intact (`forge.rs` backs both `release` and `release-pr`).

**Acceptance:** default runs release-on-push with no PR; enabling the flag opens a whole-repo release PR.

---

## Phase 6 — Monorepo subtree config (low)

- [ ] `packages_dir` (from `[project]`) scopes which subtree to operate on; skip `publish = false` (already honored). Mostly config plumbing over the existing workspace-member logic.

---

## Cross-cutting

- Every phase lands under the full gate (`skills/quality-gate.md`).
- Preserve idempotency/resumability and dependency ordering everywhere (already solid — don't regress).
- Update `.schema/latest.json` whenever `config.rs` changes.
