# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project aims to follow Semantic Versioning.


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
