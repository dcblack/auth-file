# Security Policy

## Supported versions

Until the crate reaches 1.0, security fixes are expected only on the latest minor version.

## Reporting a vulnerability

Please do not open public issues for suspected vulnerabilities. Report privately through GitHub Security Advisories when the repository is available, or by email to the maintainer listed in `Cargo.toml`.

## Security model summary

`auth` stores no plaintext file paths in record filenames. It uses a local secret and HMAC-SHA256 to derive record names from canonical paths. File contents are validated with SHA-256 and records are signed with Ed25519.

The tool does not make unauthorized files safe. It only answers: "Does this current file match a previously authorized state according to this user's local trust database?"

## Known limitations

- If the local signing key and path-HMAC key are stolen, an attacker can forge trust records for that database.
- The default database is local-user scoped, not machine-wide policy.
- macOS Touch ID support currently uses a helper binary.
- Linux authorization currently uses PAM through `sudo -v`; direct PAM/fprintd integration is a future enhancement.
