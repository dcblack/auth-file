# auth-file / `auth`

`auth` is a small command-line tool and Rust library for authorizing and validating files in scripts.

The published crate name is currently set to `auth-file` because the crate name `auth` appears to already be occupied on crates.io. The installed binary remains `auth`.

## Status

Version: `0.3.0`

This is a development implementation intended for review and platform testing.

## Security model

`auth` separates two ideas:

1. **Authorization**: a trusted user approves a database-changing action such as `--write` or `--remove`.
2. **Validation**: later checks verify that a file still matches its authorized state.

Record filenames are derived from `HMAC-SHA256(canonical_path, local_path_key)`, not a plain path hash. That avoids exposing sensitive filenames in the database and makes dictionary attacks much harder unless the local path-HMAC key is stolen.

Each authorization record stores:

- record version
- tool version
- creation timestamp
- path HMAC
- content SHA-256
- file size
- Ed25519 signature

## Supported platforms

| Platform | Status | Authorization backend |
|---|---:|---|
| macOS Tahoe | test target | Touch ID / password fallback through LocalAuthentication helper |
| Windows 11 | test target | Windows Hello through `UserConsentVerifier` |
| Ubuntu 24.04 | test target | PAM through `sudo -v` fallback |
| Other Linux | experimental | PAM through `sudo -v` fallback |

See `docs/platform-support.md` for details.

## CLI

```bash
auth --help
auth --version
auth --write  [OPTIONS] FILENAME...
auth --check  [OPTIONS] FILENAME...
auth --remove [OPTIONS] FILENAME...
```

Examples:

```bash
auth --write important-script.sh
auth --check important-script.sh
auth --remove important-script.sh
```

CI / non-interactive examples:

```bash
auth --no-platform-auth --write important-script.sh
auth --check important-script.sh
```

## Library API

Simple shell-friendly wrapper:

```rust
pub fn auth(
    action: ActionType,
    file_list: Vec<String>,
    options: AuthOptions,
) -> bool;
```

Detailed API:

```rust
pub fn auth_report(
    action: ActionType,
    file_list: Vec<String>,
    options: AuthOptions,
) -> Result<AuthReport, AuthError>;
```

## Build

```bash
cargo build --release
```

## macOS Touch ID helper

Build and install the helper:

```bash
swiftc platform/macos/auth-macos-touchid.swift -o auth-macos-touchid
install -m 0755 auth-macos-touchid /usr/local/bin/auth-macos-touchid
```

`auth` itself has no GUI. The helper asks macOS to show the normal LocalAuthentication prompt.

## Tests

```bash
cargo test --all-features
cargo test --test cli
```

## Security checks and SBOM

```bash
cargo install cargo-audit cargo-cyclonedx cargo-deny
cargo audit
cargo deny check advisories bans licenses sources
cargo cyclonedx --format json --output-file sbom.cdx.json
```

Or run:

```bash
scripts/security-checks.sh
```

## Packaging

```bash
cargo package
cargo publish --dry-run
```

See `docs/publishing-checklist.md` before publishing.

## License

Apache-2.0. See `LICENSE` and `NOTICE`.


## macOS Touch ID helper build

On macOS, `build.rs` compiles `platform/macos/auth-macos-touchid.swift` into Cargo's `OUT_DIR` and embeds that helper path into the Rust binary. The runtime lookup order is:

1. `AUTH_MACOS_TOUCHID_HELPER` environment variable
2. helper compiled by `build.rs`
3. helper installed beside the `auth` executable
4. `auth-macos-touchid` found on `PATH`

For development and CI, use `--no-platform-auth` to avoid an interactive biometric/PAM/Hello prompt.

## Integration tests

Run:

```bash
cargo test --all-targets --all-features
```

The CLI integration tests cover help/version output, writing authorization for two files, checking authorized/unauthorized/missing files, removing one authorization record, and detecting content changes.
