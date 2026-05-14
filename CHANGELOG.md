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

## [0.8.0] - 2026-05-13

### Added

- Added fallback password support for normal databases using Argon2id password hashes.
- Added encrypted key backup using an Argon2id-derived key and XChaCha20-Poly1305.
- Added one-time burner passwords for changing the fallback password if the current fallback password is unavailable.
- Added `--change-password` to rotate the fallback password and generate a new burner set.
- Added machine-binding metadata to detect database use on a different machine.

### Changed

- New normal databases now attempt to initialize fallback recovery material after key creation.
- If platform authorization is unavailable, write/remove operations can fall back to the stored fallback password when recovery is configured.

### Security Notes

- Fallback recovery data is intended to recover access to the database keys when platform authorization or local credential-store access is unavailable.
- The machine-binding check is advisory; it helps detect unexpected database movement but is not equivalent to hardware attestation.
- Burner passwords are displayed only once and are not recoverable.
