# AGENTS.md — `zen-release`

## What this is

**`zen-release`** is a config-driven release-management tool for Rust, not bound to any framework. It targets large frameworks and applications that ship a base product *and* a set of individual crates together, and it drives the whole release from one config file: custom conventional-commit prefixes and version rules, one or more Rust registries (including private ones), optional mirror publishing to crates.io, dependency-ordered publishing, and rate-safe batching.

The point is a simple release process that ships updates exactly where you want them — a private registry, crates.io, or both — without hand-rolled scripts and without exhausting crates.io publish rate limits.

It is distributed as a crate and standalone binary, and as a reusable **GitHub Action** so a consuming repo gets the whole pipeline from one `uses:` line.

`zen-release` is a rewrite and upgrade of the release-plz approach, built for flexibility — not a fork that tracks upstream. release-plz gets the generic 80% right (conventional-commit → semver, changelog via git-cliff, dependency-ordered `cargo publish`, workspace handling) but is **crates.io-centric and PR-first**; zen-release keeps what works and makes the rest config-driven. (The code started from release-plz's MIT-licensed source; its license and copyright are retained in `licenses/RELEASE-PLZ.md` until the relevant parts are rewritten. The project itself is MIT, © Jetstream Labs — see `LICENSE.md`.)

## Why the stock tools didn't fit (the motivation)

A project that publishes to a **private registry first** with **crates.io as an optional downstream mirror** breaks the assumptions of every off-the-shelf tool:

- **Registry baseline.** release-plz derives its "last released version" baseline partly from crates.io and has no clean alternative-registry publish path; its headline feature is an auto-generated release PR, which not everyone wants. `zen-release` takes the baseline from the last **git tag**, so the authoritative history is the repo, not any registry.
- **crates.io forbids a published crate from depending on an alternative-registry crate.** A crate that is self-contained on a private registry (`registry = "..."` on its internal deps) **cannot** be published to crates.io as-is. Dual-publishing therefore requires two manifest forms from one source — a transform that has to live *somewhere*, and the least-bad place is inside the release tool, not a developer's working tree.
- **crates.io rate-limits publishing** to ~30 "update existing crate" operations in a burst. A lockstep release of a few dozen crates exceeds that and 429s partway. So the crates.io path must **batch** and be **rate-aware**.

No existing tool solves these together. That is the reason this tool exists.

## The pipeline

```
Developer ──(conventional commit, push to release branch)──> repo
   │
   ▼
zen-release (GitHub Action or CLI; on push, no PR required)
   1. version: conventional commits since last tag → semver bump (lockstep or per-crate, per config)
   2. changelog: git-cliff, tag(s) per the configured tag format
   3. bump: set workspace/package versions + internal dependency versions
   4. publish → primary registry:
        - internal deps rewritten to the registry (self-contained), if configured
        - dependency-ordered, waits for the sparse index between crates
   5. tag + release, commit the bump back
   6. mirror → crates.io (optional):
        - strip the alternative-registry marker back to plain deps (crates.io rule)
        - publish in rate-safe batches, dependency-ordered
        - idempotent: skip already-published; defer/retry on 429
```

Every registry in the pipeline is defined in config; a single-registry setup simply omits the mirror step. The manifest transform + batching is always the tool's responsibility, never a developer's.

## Requirements (the spec, learned the hard way)

These are the capabilities the tool provides; most are switched on or shaped by config.

1. **Versioning is tag-based.** Baseline is the last **git tag**, not any registry. Lockstep (all crates share one version, one tag per release) and per-crate versioning are both supported via config.
2. **Configurable conventional-commit → semver rules.** Prefixes and their bump levels are config, not hardcoded (see the default ruleset below).
3. **PR is optional, not forced (defaults to off).** The release-PR config flag defaults to `false`: out of the box, release runs on push/merge to a branch with no PR. A repo that wants a release PR sets it to `true`; unlike release-plz's crates-only release PR, zen-release's PR spans the **whole repository**, not just the crates subtree. A human can also open a PR traditionally; merging it is just another push.
4. **Private-registry-primary, self-contained.** Published crates can resolve their internal deps from the primary registry, so it is usable before crates.io catches up.
5. **Optional crates.io mirror.** Same versions, internal deps stripped to plain, published in rate-safe batches. Only the tool/CI needs a crates.io token.
6. **Dependency-ordered publish** with sparse-index waits, on every registry.
7. **Idempotent + resumable.** Skip already-published versions; a partial run can be re-run to completion.
8. **Monorepo-aware.** Operate on a configured packages subtree; ignore examples and anything `publish = false`.

## Default conventional-commit rules

Prefixes are configurable. This is the default ruleset (and the one this repo uses for itself):

| type / scope | release |
|---|---|
| `breaking` | major |
| `feat` | minor |
| `fix` | patch |
| `refactor` | patch |
| `docs` | patch |
| `wip` | none |
| `chore` | none |
| scope `style` | none (scope is descriptive; a mis-typed `feat(style):` is the author's mistake) |
| scope `test` | none |

`breaking:` is a *type*, so major requires a custom major-increment rule (git-cliff: `custom_major_increment_regex = "^breaking"`); cargo's native major trigger is only `!` / `BREAKING CHANGE`. Changelog grouping is by type.

## Config (`zen-release.toml`)

One config drives version rules, every registry, self-containment, and batching. Example of a private-primary + crates.io-mirror setup:

```toml
[project]
packages_dir = "packages"          # monorepo subtree to publish; skip publish=false
lockstep   = true
tag_format = "v{{version}}"

[[registry]]
name    = "primary"            # a private registry
kind    = "kellnr"            # e.g. kellnr, or any sparse registry
index   = "sparse+https://crates.example.org/api/v1/crates/"
token   = "env:CARGO_REGISTRIES_PRIMARY_TOKEN"
self_contained = true          # rewrite internal deps to registry = "primary"

[[registry]]
name       = "crates-io"       # optional public mirror
kind       = "crates-io"
token      = "env:CARGO_REGISTRY_TOKEN"
strip_alt_registry = true      # deps → plain for crates.io
batch_size = 18                # rate-safe
batch_gap_secs = 3600
retry_on_429 = true

[bump]                          # configurable; git-cliff compatible
custom_major_increment_regex = "^breaking"
[[bump.rules]]  # feat->minor, fix/refactor/docs->patch, wip/chore->none
```

The exact shape is still open; the invariant is that one config file drives the whole pipeline.

## Distribution

- A crate (`zen-release`) and a standalone binary.
- A **composite GitHub Action** wrapping the binary, so a consuming repo needs only `uses: zen-rust/release@v1`.
- Optionally surfaced as a subcommand of a host CLI (e.g. `zen release`) where a framework wants to embed it.

## Reference deployment: Zenrust

The first real consumer, and the setup that motivated the tool. Useful as a concrete example; none of it is baked into the tool.

- **Primary — Kellnr** at `https://crates.zenrs.org` (self-hosted, no publish rate limit). Sparse index `sparse+https://crates.zenrs.org/api/v1/crates/`; cargo registry name `zenrust`; token env `CARGO_REGISTRIES_ZENRUST_TOKEN`.
- **Mirror — crates.io.** The `zenrust-*` names are already owned; crates.io crates can't be deleted, so the names are permanently theirs. Rate-limited (~30/burst), so the mirror batches and strips `registry = "zenrust"`.
- Ships lockstep 1.x semver ("forever 1.0", no perpetual 0.x); first release under this tool is `1.0.0`.
- Once monorepo publishing lands, the `zen-rust/packages` submodule can be folded back into the framework repo — that split existed only to get separate release tooling, which this tool removes the need for.

## Repo layout

- `packages/release` — the CLI binary (`zen-release`).
- `packages/release_core` — the engine: version calc, changelog, publish orchestration. **Most of our changes land here** (alt-registry publish, self-containment, crates.io mirror + batching).
- `packages/next_version` — conventional-commit → semver bump rules.
- `packages/cargo_utils`, `git_cmd` — cargo/git helpers.
- `packages/fake_package`, `test_logs` — test support.

Keep the git-cliff changelog integration and the dependency-graph ordering. Keep the release-PR (GitHub/Gitea/GitLab) surface too, but make it **opt-in via config** and generalize it from crates-only to **whole-repo** scope. The crates.io-baseline assumption is the main thing we replace.

## Open decisions

1. **MVP boundary** — first cut can shell out to the existing git-cliff/cargo paths and add only the multi-registry + batching glue; absorb more natively later.
2. **crates.io mirror placement** — inline in the run vs delegated to a server-side webhook near the primary registry.

## Working conventions

- Rust workspace; run the full gate (fmt + clippy pedantic `-D warnings` + tests, all-features and no-default-features) before committing.
- Commits follow the configured conventional-commit rules (see the default ruleset above); this repo will eventually release *itself* with this tool.
- Markdown prose: one line per paragraph, no hard wrapping.
- Do not add attribution lines to commits or PRs.
