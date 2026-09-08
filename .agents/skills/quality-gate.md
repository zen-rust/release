# Skill: quality-gate

The definition of "done" for any code change (from `AGENTS.md` working conventions). A task is not done until this is green.

## Toolchain (the Rust compiler — NOT the crate version)

The crates are version `0.0.1`; this is about `rustc`. The workspace deps (the `cargo` crate) need **rustc ≥ 1.96**. Installed stable is `rustc 1.98.1` and works. Verify:

```bash
rustc --version   # expect >= 1.96 (currently 1.98.x)
```

## Fast inner loop (per edit)

```bash
cargo check --workspace --all-targets
```

Capture the real exit code — **do not pipe cargo through `tail`/`head`** (the pipe's exit code masks cargo's). Redirect instead:

```bash
cargo check --workspace --all-targets > /tmp/zr_check.log 2>&1; echo "EXIT=$?"; tail -20 /tmp/zr_check.log
```

## Full gate (before marking a task done)

```bash
# format
cargo fmt --all --check

# lints: pedantic, warnings are errors (workspace lints already configured in Cargo.toml)
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings

# tests, both feature configurations
cargo test --workspace --all-features
cargo test --workspace --no-default-features
```

Notes:
- `default` features include `docker-tests` (needs a Docker runtime) and `all-static`. If Docker isn't available, run tests with `--no-default-features` (or a feature set that excludes `docker-tests`) and note it in `progress.md` rather than silently skipping.
- Some integration tests hit the network / a git forge; flakiness there is environmental, not a gate failure — note and re-run.

## Schema freshness

If `packages/release/src/config.rs` (or any `schemars`-derived struct) changed, regenerate and commit the schema:

```bash
cargo run --bin zen-release -- generate-schema
```

The `generate_schema` test fails if `.schema/latest.json` is stale.

## Record

After a green gate, update `status/progress.md`: check the task, bump "Gate status", add a dated Progress-log line.
