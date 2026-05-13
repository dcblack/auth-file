# auth-file / `auth`

`auth` is a small command-line tool and Rust library for authorizing and validating files in scripts.

The published crate name is currently set to `auth-file` because the crate name `auth` appears to already be occupied on crates.io. The installed binary remains `auth`.

## Status

Version: `0.7.0`

This is a development implementation intended for review and platform testing.

## Security model

`auth` separates two ideas:

1. **Authorization**: a trusted user approves a database-changing action such as `--write` or `--remove`.
2. **Validation**: later checks verify that a file still matches its authorized state.

Authorization records are now stored in SQLite at `~/.auth/auth.db` by default. File paths are stored as `HMAC-SHA256(canonical_path, local_path_key)`, not plaintext and not a plain path hash. That avoids exposing sensitive filenames in the database and makes dictionary attacks much harder unless the local path-HMAC key is stolen.

The database stores:

- record version
- tool version
- creation/update timestamps
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
mkdir -p auth-test
auth --no-platform-auth --dir ./auth-test --write important-script.sh
auth --dir ./auth-test --check important-script.sh
```

## Library API

Simple shell-friendly wrapper:

```rust
pub fn auth(
    action: ActionType,
    file_list: Vec<String>,
    options: &AuthOptions,
) -> bool;
```

Detailed API:

```rust
pub fn auth_report(
    action: ActionType,
    file_list: Vec<String>,
    options: &AuthOptions,
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


## Test-only authorization bypass

`--no-platform-auth` is intentionally restricted. It may only be used when an explicit database directory is supplied with `--dir`/`-d` and that directory basename is exactly `auth-test`. This keeps development and CI workflows convenient without making the production `~/.auth` database easy to modify without platform authorization.

Example:

```bash
echo "Hello World" > TESTFILE.txt
mkdir -p auth-test
export AUTH_OPTIONS="-d ./auth-test"
auth --no-platform-auth --write TESTFILE.txt
```

When this mode is active, `auth` emits a warning unless `--silent` is used.

## AUTH_OPTIONS

`AUTH_OPTIONS` is parsed before command-line arguments. It is intended for common options such as a test database directory:

```bash
export AUTH_OPTIONS="-d ./auth-test --color auto"
```

Command-line options are processed after `AUTH_OPTIONS`, so they can extend or override the initial options.

## Color output

Use:

```bash
auth --color auto   ...
auth --color always ...
auth --color never  ...
```

Errors are red, warnings are yellow, and passing/positive messages are green when color is enabled. `NO_COLOR` and `NOCOLOR` disable automatic color output. `--color always` overrides those variables.

## Paged help

`auth --help` uses `$PAGER` when stdout is interactive. If `$PAGER` is unset, it tries `less -R`, then `more`, and finally falls back to plain stdout. In non-interactive contexts such as tests or pipes, help is printed directly.


## SQLite storage with platform credential-store keys in v0.7.0

Version 0.7.0 is the first SQLite-backed implementation. The older v0.5.0 line is the last directory/file-record implementation. The default storage layout is:

```text
~/.auth/
  auth.db
  ed25519.signing-key
  ed25519.verifying-key
  path-hmac.key
```

On Unix-like systems the directory is set to `0700`, and private files are set to `0600`. Windows ACL tightening is still future work. SQLite is not encrypted in this release; privacy comes from path HMACs, content hashes, and signed records.


## v0.7.0 security changes

Version 0.7.0 is a clean break from the v0.5.0 directory/file record format. It does not import legacy flat-file authorization records.

Normal-use key material is now stored in the platform credential store using the Rust `keyring` crate. On macOS this maps to Keychain, on Windows to the Windows Credential Manager, and on Linux to a Secret Service-compatible backend when available. Test databases whose directory basename is exactly `auth-test` still use local file-backed keys so CI and development tests remain non-interactive.

The hidden `--no-platform-auth` option remains available only for test databases named `auth-test`; it is intentionally omitted from the user help text.
