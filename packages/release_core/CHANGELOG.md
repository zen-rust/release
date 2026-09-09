# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1](https://github.com/zen-rust/release/releases/tag/zen_release_core-v0.0.1) - 2026-09-09

### Added

- add inline crates.io mirror publish with batching, 429 retry, dep stripping
- apply self-contained registry rewrite on publish with manifest restore
- add rate-safe batching and rate-limit detection helpers for crates.io mirror
- expose breaking_always_increment_major and no_increment_regex bump rules
- thread primary registry and self-contained flag into release request

### Fixed

- use the requested registry's token when publishing (fixes mirror auth)
- allow mirror publish by setting the publish field to the mirror registry

### Other

- initial commit
