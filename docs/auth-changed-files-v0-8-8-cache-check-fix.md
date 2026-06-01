---
agent: "ChatGPT 5.5"
created: "2026-05-29T13:41:52+00:00"
---

# auth changed files: check/cache fix

## Files changed

- `src/lib.rs`
- `tests/cli.rs`
- `tests.mk`

## Summary

- Removed the `last_check_unix` metadata write from normal `--check` so check remains read-only.
- Added a CLI integration test proving `--check --cache-time=60` does not request any password.
- Updated the manual cache test so a cache created with `--cache-time=60` is reused by a later protected command that does not repeat `--cache-time`.
- Preserved the fixed `tests.mk` pattern using `if env ... command; then`.

## Validation

Run:

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
make test-cache
make test-bad-password
```

## Original queries

- Provide changed files instead of a patch.
- Fix `--check` asking for password when `--cache-time=60` is supplied.
- Keep normal `--check` read-only.
