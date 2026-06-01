---
agent: "ChatGPT 5.5"
created: "2026-05-31T20:10:56+00:00"
version: "0.1.0"
---

# v0.8.10+d secret provider fix

## Changes

- Fixed `--secret-provider=env` / `environment` to map to `SecretProvider::Env`.
- Fixed `--secret-provider=keyring` / `keys` to map to `SecretProvider::OsKeyring`.
- Preserved `prompt`.
- Added aliases:
  - `1p`, `1pw` -> `1password`
  - `bw` -> `bitwarden`
  - `environment` -> `env`
  - `keys`, `oskeyring`, `os-keyring` -> `keyring`
- Expanded CLI tests so all aliases parse without producing `unknown secret provider`.

## Validate

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Original queries

- Fix cargo check failure caused by wrong `SecretProvider` variant names.
