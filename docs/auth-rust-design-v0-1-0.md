---
agent: "ChatGPT 5.5"
created: "2026-05-12T16:22:59+00:00"
version: "0.1.0"
title: "Rust auth utility design"
---

# Rust `auth` utility design

## Original queries

- How should a small command-line `auth` utility be implemented in Rust to authorize and validate files?
- How can the existing Bash syntax be preserved while improving security?
- Can Touch ID, Windows Hello, or Linux fingerprint/PAM support be used without a GUI?
- What should the library API and CLI shape look like, including `--version`?

## Summary

The Bash prototype signs files with GPG detached signatures saved under a directory keyed by `git hash-object`.
That is useful as a sketch, but it is easy to bypass because the authorization step is not bound to a trusted OS authentication event, the signature storage directory is user-controlled, and the security policy is not explicit.

A Rust implementation should separate three concerns:

1. File integrity: cryptographic digest/signature or MAC over canonical file content.
2. User authorization: ask the operating system to authenticate the current user for write/remove operations.
3. Policy and storage: store authorization metadata safely and consistently.

Recommended shape:

```rust
pub fn auth(
    action: ActionType,
    file_list: Vec<String>,
    options: AuthOptions,
) -> bool
```

Use a `Result<AuthReport, AuthError>` internally, then convert to `bool` for shell compatibility.

## Recommended CLI

```text
auth --help
auth --version
auth --write  [OPTIONS] FILE...
auth --check  [OPTIONS] FILE...
auth --remove [OPTIONS] FILE...
```

Compatibility aliases from the Bash script:

```text
-ck, --check
-wr, --write
-rm, --remove
-d,  --dir DIR
-v,  --verbose
-q,  --quiet
-s,  --silent
-f,  --force
-h,  --help
     --version
```

Suggested new options:

```text
--auth-provider auto|none|touchid|windows-hello|pam|fprintd|password
--format json|human
--strict
--keychain
--signature-mode gpg|minisign|ed25519|hmac
```

## Recommended Rust types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Check,
    Write,
    Remove,
}

#[derive(Debug, Clone)]
pub enum AuthProvider {
    Auto,
    None,
    MacOsLocalAuthentication,
    WindowsHello,
    LinuxPam,
    LinuxFprintd,
    Password,
}

#[derive(Debug, Clone)]
pub struct AuthOptions {
    pub store_dir: Option<std::path::PathBuf>,
    pub provider: AuthProvider,
    pub force: bool,
    pub verbosity: Verbosity,
    pub strict: bool,
}

pub fn auth(
    action: ActionType,
    file_list: Vec<String>,
    options: AuthOptions,
) -> bool {
    auth_report(action, file_list, options)
        .map(|r| r.success)
        .unwrap_or(false)
}

pub fn auth_report(
    action: ActionType,
    file_list: Vec<String>,
    options: AuthOptions,
) -> Result<AuthReport, AuthError> {
    // canonicalize files
    // authenticate for Write/Remove
    // sign/check/remove per-file
    todo!()
}
```

## Security model

Do not use Touch ID or Windows Hello as a magic Boolean that directly means “trust this file.”
Use OS authentication only to unlock permission to create or remove authorization records.

For file validation, use a real cryptographic record:

- Store canonical path, file size, modification time, digest, algorithm, tool version, and signature.
- Prefer Ed25519 signatures or HMAC with a key protected by OS credential storage.
- On macOS, store the signing key in the Keychain and require LocalAuthentication to retrieve/use it.
- On Windows, use Windows Hello / UserConsentVerifier for consent, and DPAPI or Windows credential/key storage for protected secret material.
- On Linux, prefer PAM for authentication and a file-permission-protected key, or integrate with Secret Service / libsecret where appropriate.

## Platform notes

### macOS

Use LocalAuthentication through a tiny Swift helper, Objective-C FFI, or a Rust crate if mature enough.
For a production CLI, the most robust approach is usually:

```text
Rust CLI
  -> small signed macOS helper
  -> LAContext.evaluatePolicy(...)
  -> return success/failure
```

LocalAuthentication can request Touch ID or device owner authentication. Device-owner authentication allows password fallback.

### Windows

Use Windows Hello via `Windows.Security.Credentials.UI.UserConsentVerifier`.
For desktop apps, use the desktop/WinRT interop route rather than assuming a UWP-only model.

### Linux

Linux fingerprint support is uneven. The practical choices are:

1. PAM authentication, which lets the system decide whether fingerprint/password/etc. is acceptable.
2. `fprintd` / D-Bus for fingerprint-specific verification.
3. Password-only fallback.

PAM is usually better for a CLI because it matches local system policy.

## Suggested implementation phases

### Phase 1

- Port the Bash CLI syntax using `clap`.
- Implement `--version`.
- Implement file canonicalization.
- Implement digest-based check/write/remove.
- No biometrics yet; use `--auth-provider none` for local testing.

### Phase 2

- Add Ed25519 or HMAC authorization records.
- Store metadata as JSON or CBOR.
- Add structured JSON output for scripts.

### Phase 3

- Add macOS Touch ID through LocalAuthentication.
- Add Linux PAM authentication.
- Add Windows Hello consent verification.

### Phase 4

- Harden storage permissions.
- Add tests for symlinks, renamed files, modified content, deleted signatures, and mixed action syntax.
- Add shell-completion generation.

## References

- https://developer.apple.com/documentation/localauthentication/lacontext
- https://developer.apple.com/documentation/localauthentication
- https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverifier.requestverificationasync
- https://learn.microsoft.com/en-us/uwp/api/windows.security.credentials.ui.userconsentverifier
- https://learn.microsoft.com/en-us/windows/apps/develop/security/windows-hello
- https://fprint.freedesktop.org/
- https://linux.die.net/man/1/fprintd
- https://doc.rust-lang.org/book/
