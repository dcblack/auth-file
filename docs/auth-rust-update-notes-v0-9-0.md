---
agent: "ChatGPT 5.5"
created: "2026-05-31T00:00:00-05:00"
version: "0.9.0"
---

# auth-file v0.9.0 update notes

## Implemented

- Bumped crate version to `0.9.0`.
- Added `toml` dependency.
- Converted configuration parsing from shell-style `VAR=VALUE` to TOML.
- Added TOML `options = [...]` support.
- Kept `AUTH_OPTIONS` support as an optional TOML key, accepting either a string or an array of strings.
- Added `--config=` behavior to disable default configuration loading for one invocation.
- Preserved config precedence order:
  1. config file
  2. `AUTH_OPTIONS`
  3. command-line arguments
- Preserved root-directive duplicate enforcement across all layers.
- Updated config tests for TOML syntax.

## Notes

The help text now documents preferred long-option `--name=value` style for value options. Parser compatibility for existing split-style long options may still remain in a few places so existing tests/scripts do not all break at once. Full strict enforcement can be finished after the TOML configuration transition is validated.

## Validate

```bash
gmake verify
```

or directly:

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Original queries

- Move `.authrc` configuration from environment-style lines to TOML.
- Use config, then environment, then command-line precedence.
- Add `--config=""` behavior to disable default config loading.
- Prefer `--name=value` syntax for long options.
