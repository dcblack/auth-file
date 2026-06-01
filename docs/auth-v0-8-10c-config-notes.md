---
agent: "ChatGPT 5.5"
created: "2026-05-31T19:12:37+00:00"
version: "0.8.10c"
---

# auth v0.8.10c changed files

## Added

- `--config=FILE` support.
- Default `$HOME/.authrc` support when present.
- Simple config-file parsing for supported `AUTH_*` variables.
- Bash-style blank lines, comments, inline comments, whitespace, and quoted values.
- Config tests covering:
  - config-file supplied `AUTH_OPTIONS`
  - quoted values and comments
  - unknown variable rejection
  - missing explicit config file rejection
  - `AUTH_OPTIONS="--config=FILE"` redirection

## Supported config variables

- `AUTH_OPTIONS`
- `AUTH_TEST_FALLBACK_PASSWORD`
- `AUTH_TEST_FALLBACK_PASSWORD_CONFIRM`
- `AUTH_TEST_CURRENT_PASSWORD_OR_BURNER`
- `AUTH_MACOS_TOUCHID_HELPER`

## Test note

CLI integration tests now set `AUTH_CONFIG_DISABLE=1` by default so a developer's real `$HOME/.authrc` cannot make tests non-hermetic. Tests that explicitly validate config-file behavior remove that variable.

## Validation

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Original queries

- Add `.authrc` support.
- Add `--config=FILE` to redirect config loading.
- Support simple `VAR=VALUE` syntax with comments/whitespace/quotes.
