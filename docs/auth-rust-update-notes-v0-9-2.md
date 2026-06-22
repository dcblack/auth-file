# auth v0.9.2 update notes

## Scope

Configuration stabilization and diagnostics.

## Changes

- Default config file changed from `~/.authrc` to `~/.auth.toml`.
- `version = 1` is accepted as TOML configuration metadata and is not converted to a CLI option.
- Non-integer config versions are rejected.
- Unsupported config versions are rejected.
- Added protected `--show-config` to display the selected configuration source and effective settings.
- Help and README now document `~/.auth.toml`, `version = 1`, `secret_ref`, and `--show-config`.

## Validation

Run:

```bash
gmake verify
gmake tests-all
```

