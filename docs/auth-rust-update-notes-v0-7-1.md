---
agent: "ChatGPT 5.5"
created: "2026-05-12T20:00:00-05:00"
version: "0.7.1"
---

# auth Rust update notes v0.7.1

## Summary

Version 0.7.1 adds the requested tests around first-run bootstrap, database/key reuse, corrupted database handling, missing key handling, and hidden test-only authorization bypass behavior.

## Added tests

1. First-run bootstrap
   - A new `auth-test` directory starts with no database and no keys.
   - A write operation creates `auth.db`, the test signing key, the test path-HMAC key, and the public verifying key.

2. Existing database reuse
   - A second operation reuses the same test keys.
   - The test confirms the signing key and path-HMAC key bytes are unchanged.

3. Corrupted database handling
   - A bogus `auth.db` file returns a SQLite error rather than silently reinitializing or overwriting the file.

4. Missing key handling
   - If a database already contains records and the test path-HMAC key is missing, the operation fails with a key-storage error.
   - This avoids silently generating unrelated replacement keys for an existing database.

5. Hidden test-only flag behavior
   - The existing CLI help test continues to verify that `--no-platform-auth` is not shown in normal help output.

## API cleanup

- `auth` now borrows `AuthOptions`.
- `auth_report` now borrows `AuthOptions`.

## Bootstrap behavior

For a new database directory with no `auth.db`, keys are generated automatically.

For an existing database that already has records, missing key material is treated as an error rather than regenerated.

For an existing empty database, key creation remains allowed. This avoids a bad first-run state if platform authorization fails after database creation but before keys are provisioned.

## Suggested validation

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```
