---
title: auth storage model
version: 0.6.0
agent: ChatGPT 5.5
created: 2026-05-12T22:45:00-05:00
---

# Storage model

Version 0.6.0 replaces per-file JSON authorization records with a single SQLite database.

Default layout:

```text
~/.auth/
  auth.db
  ed25519.signing-key
  ed25519.verifying-key
  path-hmac.key
```

## What SQLite stores

The `records` table stores one row per authorized canonical path. It does not store plaintext paths.

Columns include:

- `path_hmac_sha256`
- `content_sha256`
- `size`
- `version`
- `tool`
- `created_unix`
- `updated_unix`
- `signature`

## What is still outside SQLite

The local signing key and path-HMAC key are still stored as separate private files in v0.6.0. A later release should move these secrets to platform-native secure storage:

- macOS Keychain
- Windows DPAPI / Credential Manager
- Linux Secret Service / libsecret

## Security limitations

SQLite is not encrypted in v0.6.0. If an attacker has full same-user access, they can copy the database and local key files. The design is intended to reduce casual disclosure, accidental exposure, and unauthorized record modification, not to defeat malware already running as the user.
