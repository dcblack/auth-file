---
agent: "ChatGPT 5.5"
created: "2026-05-14T01:52:58+00:00"
version: "0.8.1"
---

# Auth v0.8.1 update notes

## Summary

This update removes the runtime `--no-platform-auth` CLI bypass and moves CLI integration tests to the fallback-password path.

## Changed

1. `auth_report()` now initializes database/key/recovery material before write/remove authorization fallback is needed.
2. `--no-platform-auth` is removed from the CLI parser.
3. `--change-password` remains wired through the CLI and always prints burner passwords even if routine output is silent.
4. `auth-test` integration tests use test-only password environment variables:
   - `AUTH_TEST_FALLBACK_PASSWORD`
   - `AUTH_TEST_FALLBACK_PASSWORD_CONFIRM`
   - `AUTH_TEST_CURRENT_PASSWORD_OR_BURNER`
5. Windows `AUTH_OPTIONS` path parsing preserves backslashes outside quoted strings.
6. `.gitignore` was replaced with the uploaded file so local developer directories are ignored.

## Important test behavior

The `AUTH_TEST_*` variables are honored only when the database directory basename is exactly `auth-test`.

## Suggested validation

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Original queries

- Make the larger changes needed after removing `--no-platform-auth`.
- Update tests to use the fallback-password mechanism.
- Use the current uploaded source files as context.

## References

- https://docs.rs/rpassword
- https://doc.rust-lang.org/cargo/commands/cargo-test.html
- https://rust-lang.github.io/rust-clippy/
