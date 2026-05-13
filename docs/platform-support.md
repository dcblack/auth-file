---
title: auth platform support
version: 0.6.0
agent: ChatGPT 5.5
created: 2026-05-12T16:55:00-05:00
---

# Platform support

| Platform | Status | Authorization backend | Notes |
|---|---:|---|---|
| macOS Tahoe | Planned/test target | Touch ID / password fallback through LocalAuthentication helper | Build `platform/macos/auth-macos-touchid.swift` and place `auth-macos-touchid` on `PATH`. |
| Windows 11 | Planned/test target | Windows Hello through `UserConsentVerifier` | Requires compatible Windows Hello setup. |
| Ubuntu 24.04 | Planned/test target | PAM via `sudo -v` fallback | Fingerprint may work if PAM is configured to use fingerprint auth. Direct fprintd support is future work. |
| Other Linux | Experimental | PAM via `sudo -v` fallback | Depends on local PAM/sudo policy. |

## Non-interactive mode

CI should use:

```bash
auth --no-platform-auth --write file.txt
auth --check file.txt
```

or:

```bash
auth --force --write file.txt
```

`--force` bypasses platform authorization for database-changing actions and should not be used as the normal human authorization path.


## Key storage beginning in v0.7.0

Normal-use signing and path-HMAC secrets are provisioned automatically when a new database is initialized. No separate user setup command is required. The platform credential store is used through the `keyring` crate.

Test-only databases named `auth-test` keep local file-backed test keys so automated tests can run without touching the user's real credential store.
