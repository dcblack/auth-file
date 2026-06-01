---
agent: "ChatGPT 5.5"
created: 2026-05-12T23:34:34+00:00
version: "0.7.0"
---

# auth Rust v0.7.0 update notes

## Decisions recorded

1. v0.6.0 is a clean break from v0.5.0. No flat-file authorization records are imported.
2. `--no-platform-auth` remains test-only and is hidden from normal help output.
3. Normal-use key material moves out of the `.auth` directory and into the platform credential store.

## Implementation summary

- Added the lint attributes to both `src/lib.rs` and `src/main.rs`.
- Bumped the crate version to `0.7.0`.
- Added `CHANGELOG.md`.
- Added `keyring = "2.3"` as a dependency.
- Normal database directories now provision signing and path-HMAC keys through the platform credential store.
- Test database directories named exactly `auth-test` continue to use local test key files so CI and development runs remain non-interactive.
- `--force` no longer bypasses platform authorization.
- `--no-platform-auth` is still parsed for tests, but it is intentionally omitted from the user help text.

## Validation commands

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo audit
cargo deny check
cargo geiger
```

## Important caveat

I could not compile this package in my environment because Cargo is not installed here. Please validate in RustRover. The keyring API is targeted at the stable `keyring` 2.x API style.

## References

- https://docs.rs/keyring
- https://crates.io/crates/keyring
- https://developer.apple.com/documentation/security/keychain_services
- https://learn.microsoft.com/en-us/windows/win32/seccng/cng-dpapi
- https://specifications.freedesktop.org/secret-service/latest/
- https://keepachangelog.com/en/1.1.0/
- https://semver.org/

## Original queries

- Make v0.6.0 a clean break with no v0.5.0 flat-file migration.
- Keep `--no-platform-auth` test-only and hide it from normal documentation/help.
- Move keys into platform-backed storage with simple automatic setup for new databases.
