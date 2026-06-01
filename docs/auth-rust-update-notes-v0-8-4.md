---
agent: "ChatGPT 5.5"
created: "2026-05-15T10:11:23+00:00"
version: "0.1.0"
---

# auth-file v0.8.4 implementation notes

## Summary

Implemented v0.8.4 as a draft package based on the v0.8.3 baseline.

## Changes

- Added `--request-password` to force Auth password / burner authorization.
- Added `--show-dir` to display protected storage paths after authorization.
- Added `--stats` to display the authorized entry count, most recent write time, and most recent check time after authorization.
- Changed `--cache-time` to require `--cache-time=SECONDS` syntax.
- Kept the `--cache-time` hard maximum at 120 seconds.
- Added SQLite `metadata` table.
- Added `last_write_unix` and `last_check_unix` metadata updates.
- Bumped schema version to 5.
- Alphabetized the help options.
- Added integration tests for the new CLI behavior.
- Updated README and CHANGELOG notes.

## Validation

This environment does not have Cargo installed, so validate locally:

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Original queries

- Implement v0.8.4 based on the current v0.8.3 state.
- Add alphabetized help/documentation.
- Add `--show-dir` requiring authorization.
- Require `--cache-time=SECONDS` syntax.
- Add `--stats` requiring authorization.
- Preserve `--request-password` and cache behavior for testing/CI.

## References

- https://www.sqlite.org/lang_createtable.html
- https://www.sqlite.org/pragma.html#pragma_user_version
- https://doc.rust-lang.org/cargo/commands/cargo-test.html
- https://doc.rust-lang.org/clippy/
