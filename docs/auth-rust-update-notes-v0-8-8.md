---
agent: "ChatGPT 5.5"
created: "2026-05-26T17:02:15+00:00"
version: "0.8.8"
---

# auth v0.8.8 update notes

## Fixed

- Authorization cache entries created with `--cache-time=SECONDS` are now honored by later protected commands even when those later commands do not repeat `--cache-time`.
- Tampered, expired, or machine-mismatched authorization cache entries are cleared and ignored.
- Help text keeps short authorization footnotes instead of repeating long authorization wording.

## Added

- CLI integration test proving a cache created once is reused by a later command without repeating `--cache-time`.
- CLI integration test proving a tampered authorization cache is ignored.

## Security note

The cache entry remains MAC-protected with the database-specific HMAC secret. An attacker who can edit `auth.db` but cannot access the key material cannot validly extend the cache lifetime.

This does not prevent use of the cache during the intentionally valid time window. For that reason, the default remains `0` seconds and the maximum remains `120` seconds.

## Validation

Please run:

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

I could not run Cargo in the sandbox because the `cargo` executable was not executable in this environment.

## Original queries

- Build v0.8.8 from the uploaded baseline.
- Clean up `--help` authorization wording with footnotes.
- Fix `--cache-time` so a cache created once is honored by later commands.
- Secure cached authorization against timestamp/database modification.
