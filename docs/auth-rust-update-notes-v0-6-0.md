---
agent: "ChatGPT 5.5"
created: "2026-05-12T22:55:00-05:00"
version: "0.6.0"
---

# auth Rust update notes v0.6.0

## Summary

Version 0.6.0 is the first SQLite-backed implementation. Version 0.5.0 should be treated as the final directory/file-record design.

## Storage change

The default layout is now:

```text
~/.auth/
  auth.db
  ed25519.signing-key
  ed25519.verifying-key
  path-hmac.key
```

The SQLite database stores authorization records. It does not store plaintext file paths. The path identity is still `HMAC-SHA256(canonical_path, path-hmac.key)`.

## SQLite schema

The `records` table stores:

- path HMAC
- content SHA-256
- file size
- record version
- tool version
- created timestamp
- updated timestamp
- Ed25519 signature

Records are inserted or updated with an SQLite upsert.

## Security properties

This version improves storage manageability and reduces loose-file tampering by consolidating records into a transactional SQLite database. It still relies on cryptographic checks for integrity:

- path privacy: HMAC-SHA256 with a local secret
- content validation: SHA-256
- record tamper detection: Ed25519 signature

SQLite is not encrypted in v0.6.0. Platform-native secure key storage remains future work.

## Platform permissions

On Unix-like systems:

- database directory: `0700`
- private files: `0600`

Windows ACL hardening is still future work.

## Tests changed

Existing unit and integration tests were updated to expect `auth.db` instead of per-record JSON files. The tests still cover:

- help
- version
- write two files
- check authorized, unauthorized, and missing files
- remove one authorization
- content-change detection
- `AUTH_OPTIONS`
- restricted `--no-platform-auth`
- color behavior

## Validation note

Cargo is not installed in this execution environment. Please validate locally with:

```bash
cargo fmt --all
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo audit
cargo cyclonedx --format json --output-file sbom.cdx.json
```
