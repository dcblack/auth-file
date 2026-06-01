## 0.9.1

- Updated `--help` to document TOML configuration, supported config keys, current environment variable support, and authorization footnotes.
- Reworked `README.md` to remove stale fixed-version references and describe the current security model, configuration flow, Auth password recovery, root directives, secret providers, and test workflow.
- Added clarifying source comments around configuration layering, read-only checks, authorization cache integrity, root-relative path identity, and recovery/burner encryption.

## 0.9.0

- Added structured TOML keys such as `cache_time`, `dir`, `root_dir`, `default_root`, `request_password`, `secret_provider`, and `color`.
- Added `shadow-rs` build metadata for expanded `--version` output.
- Introduced an `EffectiveConfig` assembly stage for config/env/CLI layering.
- Changed auth configuration files from shell-style `VAR=VALUE` lines to TOML.
- Added TOML `options = [...]` support for auth options that are applied before `AUTH_OPTIONS` and command-line arguments.
- Added `--config=` support to disable default configuration loading for a single invocation.
- Added `toml` dependency for structured config parsing.
- Updated config-file tests for TOML syntax.

## 0.8.10c

- Added `--config=FILE` support for auth configuration files.
- Added default `$HOME/.authrc` support when present.
- Added config parsing for supported `AUTH_*` variables with Bash-style comments, whitespace, and quoted values.
- Added tests for config-file supplied `AUTH_OPTIONS`, quoted/commented config values, missing config files, unknown config variables, and `AUTH_OPTIONS` redirection to `--config=FILE`.

## 0.8.10

- Added additional regression tests for read-only `--check`, cache semantics, and `setup.profile` tamper detection.
- Added WSL2-friendly fallback from unavailable OS keyring storage to the prompt/Auth-password provider when appropriate.

## 0.8.9

- Added `--secret-provider` support with providers: prompt, env, os-keyring, 1password, bitwarden.
- Default provider changed to `prompt`.
- Added WSL2-friendly behavior by avoiding implicit keyring dependency.
- Added provider parsing tests.

# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project aims to follow Semantic Versioning.



## [0.8.8] - 2026-05-26

### Fixed

- Cached authorization is now honored by later protected commands even when those later commands do not repeat `--cache-time=SECONDS`.
- Tampered, expired, or machine-mismatched authorization cache records are now cleared and ignored.

### Added

- Added CLI tests for cross-command cache reuse and tampered cache rejection.


## [0.8.7] - 2026-05-18

### Added

- Added `--default-root` to explicitly select default full-path file identity.
- Added duplicate root-directive detection across `AUTH_OPTIONS` and command-line arguments.
- Added CLI tests for `--default-root`, `--root-dir=PATH`, duplicate root directives, `AUTH_OPTIONS`, and implicit default-root behavior.

### Changed

- Root directives are now strict: `--root-dir=PATH` and `--default-root` may be specified at most once total. A second root directive reports `Error: Attempt to specify root directory more than once.`


## [0.8.6] - 2026-05-16

### Added

- Added standard age-format encrypted burner password file output at `auth-burners.age`.
- Added documentation telling users to install `rage` from crates.io with `cargo install rage` to decrypt burner files.

### Changed

- Burner passwords are no longer printed directly to the terminal by default.
- Auth password rotation now reports the encrypted burner file path instead of dumping recovery secrets into terminal scrollback.

### Security

- Reduces exposure of burner passwords through terminal scrollback, logs, screen sharing, and remote shells.


## [0.8.5] - 2026-05-15

### Added

- Added `--root-dir=PATH` to authorize/check/remove files using a canonical root-relative path identity.
- Added support for `AUTH_OPTIONS` to provide `--root-dir=PATH`.
- Added tests showing a rooted authorization can validate the same relative file under a different root.

### Notes

- Machine information is not part of file path authorization hashes. Machine data is used only for recovery/cache metadata, so `--no-machine-lock` was not added in this version.

## [0.8.4] - 2026-05-15

### Added

- Added `--request-password` to force Auth password / burner authorization.
- Added `--show-dir` to display protected storage paths after authorization.
- Added `--stats` to display entry count plus most recent write/check times after authorization.
- Added metadata storage for `last_write_unix` and `last_check_unix`.

### Changed

- `--cache-time` now requires `--cache-time=SECONDS` syntax.
- Help options are alphabetized.
- Schema version bumped to 5.

## [0.8.3] - 2026-05-14

### Changed

- Changed interactive recovery prompts from “fallback password” wording to “Auth password”.
- Linux and WSL now skip the incomplete `sudo -v` platform authorization path and use Auth password fallback instead.
- Platform authorization fallback now reports that Auth password fallback is being used before prompting.

### Fixed

- Burner passwords can now authorize write/remove fallback exactly once, like the normal Auth password.

## [0.8.2] - 2026-05-14

### Added

- Added `--cache-time SECONDS` with a hard CLI limit of 120 seconds.
- Added a signed short-lived authorization cache in SQLite to reduce repeated platform prompts during scripted write/remove workflows.

### Changed

- Bumped SQLite schema version to 4 for the authorization cache table.

## [0.8.1] - 2026-05-13

### Changed

- Removed `--no-platform-auth` from the CLI.
- Reworked first-run bootstrap so fallback recovery material is initialized before write/remove authorization fallback is needed.
- CLI integration tests now use the fallback-password mechanism through `auth-test`-only `AUTH_TEST_*` variables.

### Fixed

- Fixed Windows `AUTH_OPTIONS` parsing so backslashes outside quotes remain literal path separators.
- Kept one-time burner password output visible even when routine CLI output is silent.

## [0.7.1] - 2026-05-12

### Added

- Added tests for first-run bootstrap, existing database key reuse, corrupted database handling, and missing test key handling.

### Changed

- `auth` and `auth_report` now borrow `AuthOptions` rather than consuming it.
- Existing databases now require their previously provisioned key material; missing keys are reported as errors instead of silently creating unrelated replacement keys.

### Fixed

- Cleaned up Clippy errors caused by strict lint gates in the CLI and library.

## [0.7.0] - 2026-05-12

### Added

- Platform credential-store key provisioning using the `keyring` crate.
- Automatic key creation when a new non-test database directory is initialized.
- Hidden test-only `--no-platform-auth` behavior retained for `auth-test` directories.
- Crate-level lint attributes in both `src/lib.rs` and `src/main.rs`.

### Changed

- v0.6.0+ remains a clean break from v0.5.0 flat-file authorization records.
- `--force` no longer bypasses platform authorization.
- Help text no longer documents the test-only authorization bypass.

### Security

- Normal signing and path-HMAC keys are no longer stored as loose files in regular database directories.
- Test key files are limited to explicitly named `auth-test` database directories.

## [0.6.0] - 2026-05-12

### Added

- SQLite-backed authorization record storage.

### Changed

- Replaced the v0.5.0 loose directory/file-record storage model.

## [0.5.0] - 2026-05-12

### Added

- Guard rails around `--no-platform-auth`.
- `AUTH_OPTIONS` support.
- Colored output and pager-backed help.

## [0.4.0] - 2026-05-12

### Added

- Build-script support for the macOS Touch ID helper.
- Integration tests for help, write, check, missing file, and remove flows.

## [0.8.0] - 2026-05-13

### Added

- Added auth password support for normal databases using Argon2id password hashes.
- Added encrypted key backup using an Argon2id-derived key and XChaCha20-Poly1305.
- Added one-time burner passwords for changing the auth password if the current auth password is unavailable.
- Added `--change-password` to rotate the auth password and generate a new burner set.
- Added machine-binding metadata to detect database use on a different machine.

### Changed

- New normal databases now attempt to initialize fallback recovery material after key creation.
- If platform authorization is unavailable, write/remove operations can fall back to the stored auth password when recovery is configured.

### Security Notes

- Fallback recovery data is intended to recover access to the database keys when platform authorization or local credential-store access is unavailable.
- The machine-binding check is advisory; it helps detect unexpected database movement but is not equivalent to hardware attestation.
- Burner passwords are displayed only once and are not recoverable.
