# zen-release

Config-driven release management for Rust — ship base code and individual crates together, to the registries you choose, without hand-rolled scripts or crates.io rate-limit pain.

`zen-release` is a rewrite and upgrade of the release-plz approach, built for flexibility and not bound to any framework. It targets large frameworks and applications that publish a base product *and* a set of crates in lockstep, and it drives the whole release from one config file (`zen-release.toml`).

## What it does

- **Tag-based versioning.** The release baseline is the last git tag, not a registry — your repo is the source of truth.
- **Configurable conventional-commit rules.** Commit prefixes and their bump levels (major/minor/patch/none) are config, not hardcoded.
- **Multi-registry publishing.** Publish to one or more registries, including private ones (e.g. Kellnr), with dependency-ordered publishing and sparse-index waits.
- **Self-contained private crates.** Internal dependencies can be rewritten to resolve from your primary registry, so it's usable before crates.io catches up.
- **Optional crates.io mirror.** Same versions, internal deps stripped to plain, published in rate-safe batches — idempotent and resumable, with 429 back-off.
- **Optional release PR.** Off by default; when enabled, the release PR spans the whole repository, not just the crates subtree.
- **Monorepo-aware.** Operates on a configured crates subtree and skips `publish = false`.

## Status

Early and in active development. The engine (version calculation, changelog via git-cliff, dependency-ordered publishing) is working; the multi-registry, self-containment, and crates.io-mirror-batching layers are being built out. Expect the config schema to change.

## Distribution

- A crate and standalone binary (`zen-release`).
- A composite GitHub Action, so a consuming repo needs only `uses: zen-rust/release@v1`.

## Configuration

Configuration lives in `zen-release.toml` (or `.zen-release.toml`) at the repo root. The JSON schema is published at [`.schema/latest.json`](.schema/latest.json).

## License

Licensed under the MIT License ([LICENSE.md](LICENSE.md)), © Jetstream Labs, LLC. Portions of the code originate from [release-plz](https://github.com/release-plz/release-plz), whose MIT license and copyright are retained in [licenses/RELEASE-PLZ.md](licenses/RELEASE-PLZ.md).
