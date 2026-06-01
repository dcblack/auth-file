---
agent: "ChatGPT 5.5"
created: "2026-06-01T13:10:52+00:00"
version: "0.9.1+a"
---

# v0.9.1+a secret-ref fix

## Changed files

- `src/lib.rs`
- `src/main.rs`
- `tests/cli.rs`
- `CHANGELOG.md`

## Changes

- Added `AuthOptions.secret_ref`.
- Added `--secret-ref=REF` parsing.
- Added TOML `secret_ref = "..."` support.
- Added basic CLI tests so `secret_ref` does not regress into an unsupported config key.
- Updated provider lookup plumbing so external providers can receive the configured reference.

## Notes

For external providers, `REF` may include `{name}` as a placeholder for the internal auth secret name. For example:

```text
op://Private/auth-file/{name}
```

If no placeholder is present, the reference is passed to the provider unchanged.

## Original queries

- Fix missing `--secret-ref` implementation and TOML `secret_ref` support.
