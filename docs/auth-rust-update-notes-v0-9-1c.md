# v0.9.1+c config-order/test isolation update

Created: 2026-06-01T15:55:28

## Changes

- Updated `tests.mk` so manual/system tests explicitly disable default config loading by putting `--config=` in `AUTH_OPTIONS`.
- This keeps manual tests standalone even when the developer has a real `~/.authrc`.
- Added a guard in `src/main.rs` so a loaded TOML config file cannot itself specify `--config` via `options`/`AUTH_OPTIONS`.
- This makes config selection happen before loading any config and prevents nested/ambiguous config redirects.

## Validate

```bash
gmake verify
gmake tests-all
```
