---
agent = "ChatGPT 5.5"
created = 2026-05-13T14:11:00-05:00
---

# auth-file v0.8.0 update notes

This version adds a fallback-password recovery path for cases where platform authorization is unavailable or the platform credential store cannot supply existing database keys.

## What changed

- Added Argon2id fallback password hashes.
- Added encrypted key backup with an Argon2id-derived key and XChaCha20-Poly1305.
- Added ten one-time burner passwords, generated as 16-character alphanumeric passwords.
- Added `--change-password`.
- Added advisory machine-binding metadata.

## Intended behavior

When a normal database is established, `auth` prompts for a fallback password, validates basic strength, generates burner passwords, and asks the user to save them with the database path in a password manager.

If platform authorization later fails, the tool can ask for the fallback password.

If a database is moved to a different machine and the platform key store lacks the database keys, the fallback password can restore the keys to the local credential store. The user should then run `auth --change-password --dir <database-dir>` to create a fresh fallback password and burner set for the new machine.

## Limitations

The machine binding is advisory, not hardware attestation. A stronger version would use platform-specific hardware-backed keys or attestation APIs.
