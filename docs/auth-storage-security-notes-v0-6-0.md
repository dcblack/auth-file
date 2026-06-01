---
agent: "ChatGPT 5.5"
created: 2026-05-12T16:00:00-05:00
version: "0.6.0"
title: "auth storage security notes"
---

# auth storage security notes v0.6.0

## Original queries

- How should the `.auth` directory contents be protected?
- Would a database be a better storage design for a security-oriented file authorization tool?
- How far can this design go toward protecting sensitive assets?

## Short answer

A database is a better next step, but not because SQLite is magically secure. It is better because it gives `auth` transactional updates, schema versioning, fewer loose files, easier integrity checks, and a cleaner path toward encryption.

The strongest design is:

1. Store authorization records in SQLite.
2. Keep paths private using HMAC-SHA256, not plain hashes.
3. Store file digests and metadata.
4. Sign or MAC each record.
5. Store the master secret in platform-native secure storage.
6. Use biometric/platform authorization only for write/remove/admin operations.
7. Validate files non-interactively where possible.

## Proposed storage layout

```text
~/.auth/
  auth.db
  auth.db-shm
  auth.db-wal
  config.toml
```

Permissions should still be strict:

```text
~/.auth          0700
~/.auth/auth.db 0600
```

On Windows, use a user-only ACL rather than Unix mode bits.

## Recommended SQLite tables

```sql
CREATE TABLE schema_version (
    version INTEGER NOT NULL
);

CREATE TABLE records (
    id INTEGER PRIMARY KEY,
    path_hmac BLOB NOT NULL UNIQUE,
    content_hash BLOB NOT NULL,
    file_size INTEGER NOT NULL,
    canonical_hint TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    auth_version TEXT NOT NULL,
    record_mac BLOB NOT NULL
);

CREATE INDEX idx_records_path_hmac ON records(path_hmac);
```

`canonical_hint` should be optional and disabled by default, because even partial path hints can leak sensitive information.

## Secrets

Use separate keys for different purposes:

- `path_hmac_key` for HMAC(path)
- `record_mac_key` for authenticating database records
- optional `db_encryption_key` if encrypted SQLite is later added

These keys should not live in the database.

## Platform-native secret storage

macOS:

- Store keys in Keychain.
- Gate key creation/export/destruction behind Touch ID / device-owner authentication.

Windows:

- Store keys using DPAPI or Windows Credential Manager.
- Gate sensitive operations behind Windows Hello where available.

Linux:

- Prefer Secret Service / libsecret where available.
- Fall back to a local key file only with strict permissions and loud warnings.

## Threat model boundaries

This design can protect against:

- casual inspection of `.auth`
- accidental disclosure of sensitive file paths
- unauthorized database edits
- stale or modified files being treated as authorized
- scripts accidentally trusting the wrong file

It cannot fully protect against:

- a compromised user account
- malware running as the same user
- a malicious root/admin user
- live process memory inspection by privileged code
- a user intentionally bypassing their own tool

## Recommended next development step

Move from loose files to SQLite first, but keep the current HMAC path design. Then add record MACs. After that, add platform secure storage for the HMAC/MAC keys.

## References

- https://sqlite.org/index.html
- https://sqlite.org/wal.html
- https://sqlite.org/transactional.html
- https://docs.rs/rusqlite
- https://developer.apple.com/documentation/security/keychain_services
- https://learn.microsoft.com/en-us/windows/win32/seccng/cng-dpapi
- https://specifications.freedesktop.org/secret-service-spec/latest/
