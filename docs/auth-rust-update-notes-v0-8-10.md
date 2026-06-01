---
agent: "ChatGPT 5.5"
created: "2026-05-29T18:14:10+00:00"
version: "0.8.10"
---

# auth-file v0.8.10 changed files

## Purpose

Version 0.8.10 is intentionally focused on tests and validation hardening before documentation cleanup in 0.8.11 and the larger `src/lib.rs` refactor in 0.9.0.

## Included files

- `Cargo.toml`
- `CHANGELOG.md`
- `src/lib.rs`
- `tests/cli.rs`
- `tests.mk`

## Changes

- Bumped crate version to `0.8.10`.
- Added regression tests that reinforce:
  - ordinary `--check` remains read-only and does not require Auth password
  - `--check --cache-time=60` still does not request Auth password
  - `--request-password --check` still does not request Auth password
  - changed `setup.profile` files are rejected
- Added manual `tests.mk` targets for:
  - check-no-auth behavior
  - changed `setup.profile` rejection
- Added WSL2-friendly fallback behavior:
  - if the OS keyring/Secret Service path fails and prompt fallback is appropriate, `auth` falls back to the Auth-password provider instead of immediately failing with a keyring error.
  - This is intended to address WSL2 Ubuntu cases such as: `SS error: result not returned from SS API`.

## Validation

Run:

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
make validate
```

Then test manually on WSL2 with the default provider and with:

```bash
auth --secret-provider=prompt --request-password --write FILE
auth --check FILE
```

## Original queries

- Implement 0.8.10.
- Add explicit tests before future refactoring.
- Address WSL2 keyring/Secret Service failure.
