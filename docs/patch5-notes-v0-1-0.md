---
agent: "ChatGPT 5.5"
created: "2026-05-29T10:23:20+00:00"
version: "0.1.0"
---

# patch5 notes

This patch fixes the `--check`/cache behavior and one `tests.mk` shell syntax issue.

## Changes

- `--check` no longer updates `last_check_unix`, keeping normal checks read-only.
- Existing cached authorization is checked even when the current command does not specify `--cache-time`.
- `tests.mk` no longer uses invalid shell syntax of the form `VAR=value if command; then`.
- Manual cache test now verifies the second command does not repeat `--cache-time`.
- Added CLI integration tests:
  - `check_with_cache_time_does_not_request_password`
  - `authorization_cache_is_reused_without_repeating_cache_time`

## Apply

```bash
git apply patch5.patch
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Original queries

- Provide patch5.
