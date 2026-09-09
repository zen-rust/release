# Decision log

Locked decisions. Don't re-litigate; if one changes, edit it here with the date and reason.

## Naming & structure

- **Product / brand:** `zen-release` (tool). Framework it was born from: **Zenrust** (org `zen-rust`, repo `zen-rust/release`).
- **Directories:** `packages/` (not `crates/`). First-party crate dirs: `packages/release`, `packages/release_core`.
- **Crate (package) names:** `zen_release` (binary crate) and `zen_release_core` (lib). *Directories dropped the `zen_` prefix; package names kept it.*
- **Helper crates published under `zen_` names too** (to own them on a registry): packages `zen_cargo_utils`, `zen_git_cmd`, `zen_next_version`, `zen_test_logs`, `zen_fake_package` — **folder names and imports unchanged**. Achieved via `package = "zen_…"` on each internal dependency (import key stays the old name) plus `[lib] name = "next_version"` on next_version (its own tests self-import). Every crate also carries a copy of root `LICENSE.md` for registry packaging.
- **Binary / command:** `zen-release` (via `[[bin]] name` on the `zen_release` package, so the branded command survives the crate rename).
- **Config file:** `zen-release.toml` (and `.zen-release.toml`).
- **Env vars:** `ZEN_RELEASE_*` (e.g. `ZEN_RELEASE_LOG`, `ZEN_RELEASE_NO_ANSI`, `ZEN_RELEASE_TOKEN`).
- **Registry name (Zenrust reference deployment):** cargo registry `zenrust`, index `crates.zenrs.org`, token env `CARGO_REGISTRIES_ZENRUST_TOKEN`. This is an *example consumer*, not baked into the tool.

## Positioning

- zen-release is a **rewrite/upgrade** of the release-plz approach — **not a fork** that tracks upstream. AGENTS.md reflects this.
- Zenrust is the *first consumer / reference deployment*, documented as an example; none of it is hardcoded.

## Licensing

- Project license: **MIT © Jetstream Labs, LLC** (`LICENSE.md`); `Cargo.toml` `license = "MIT"`.
- release-plz's MIT license/copyright (© 2022 Marco Ieni) retained as attribution in `licenses/RELEASE-PLZ.md` while code remains derived.
- Upstream `github.com/release-plz/release-plz` issue/PR/compare links and inherited `CHANGELOG.md` histories are **preserved on purpose** (so we can pull upstream issues/PRs to work through). Not to be scrubbed.
- Open: add a `LICENSE-APACHE` only if we decide to dual-license (currently MIT-only).

## Versioning

- All first-party crates start at **`0.0.1`**; root `workspace.package.version = "0.0.1"`.
- Baseline for bumps = **last git tag** (not any registry). Lockstep is the intended default.
- Bump rules = **Jetstream ruleset** (breaking→major via `^breaking`, feat→minor, fix/refactor/docs→patch, wip/chore/style/test→none). Configurable.

## Release flow

- **Release runs on push** by default; **release PR is opt-in, defaults to `false`**, and when enabled spans the **whole repository** (not just the crates subtree).

## CI / distribution

- GitHub Actions workflows are parked in repo-root `workflows/` (**out of `.github/`, so nothing runs**) until we're ready.
- Docker images publish to **GHCR** under the repo's account (`ghcr.io/${{ github.repository_owner }}/zen-release`), auth via `GITHUB_TOKEN`. No Docker Hub.
- Distribution: crate + binary + composite GitHub Action (`uses: zen-rust/release@v1`).
- `zen-release` **0.0.0 placeholder** reserved on crates.io (name available; publish is the user's to run).
- Repo self-release workflow still `uses: release-plz/action@<sha>` as a bootstrap until our own action ships.

## Publish-path decisions (2026-09-07, resolved with user)

- **Self-contained application → in-place with guaranteed restore.** Implemented (`self_contained.rs`): rewrite the internal deps in the (ephemeral CI) checkout just before publishing, publish with forced `--allow-dirty`, and restore the original manifest bytes via a `ManifestBackup` RAII guard on any scope exit (return/error/panic). This meets the user's goal — the repo is never left modified — while reusing the existing in-place publish path, avoiding threading temp manifest paths through the whole call chain (simpler, lower risk than a full temp-copy). Revisit if the mirror flow (Phase 3) needs both self-contained and stripped forms in one run. (User deferred the call.)
- **crates.io mirror → inline** in the release run (not a server-side webhook). crates.io allows a burst (~30 publishes) then throttles (~1/min); with `retry_on_429` the tool self-paces when throttled, and `batch_size`/`batch_gap_secs` tune it. Long mirror runs (dozens of crates) are acceptable.
- **Pre-1.0 = patch-only.** Keep `^breaking`→major and `features_always_increment_minor` OFF. With those off on 0.x, feat/fix/refactor/docs all bump patch; only a `breaking:` commit would bump minor — so patch-only holds as long as we don't author `breaking:` commits (which we're already avoiding). Revisit at 1.0.

## Registries (to be built — Phase 1)

- Config gets a `[[registry]]` array: one **primary** + optional **mirror(s)**. Keys (proposed, to confirm): `name`, `kind`, `index`, `token`, `self_contained`, `strip_alt_registry`, `batch_size`, `batch_gap_secs`, `retry_on_429`.
