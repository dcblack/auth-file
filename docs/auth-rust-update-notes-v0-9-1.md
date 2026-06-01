# auth v0.9.1 documentation update

This update is documentation/comment focused.

## Changed

- Updated top-level help text to document TOML configuration, supported config keys, current environment variable support, root directives, secret providers, and authorization footnotes.
- Reworked `README.md` to remove stale fixed-version references and describe the current security model, configuration flow, Auth password recovery, root-relative identity, secret providers, authorization cache, and test workflow.
- Added source comments around config/env/CLI layering, TOML parsing, read-only checks, authorization cache MAC protection, root-relative file identity, and encrypted burner-file behavior.
- Bumped crate version to `0.9.1`.

## Notes

- No intended behavior changes are included beyond removing duplicate no-op statements found while commenting the code.
- `--check` remains read-only and should not require authorization.
- `--config=` remains the preferred way to disable config loading for one invocation.
