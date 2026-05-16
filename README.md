# auth-file / `auth`

`auth` is a small command-line tool and Rust library for authorizing and validating files in scripts.

The published crate name is currently set to `auth-file` because the crate name `auth` appears to already be occupied on crates.io. The installed binary remains `auth`.

## Status

Version: `0.7.0`

This is a development implementation intended for review and platform testing.

## Auth password recovery

Normal databases can establish an auth password and ten one-time burner passwords. The auth password is stored as an Argon2id password hash. A backup copy of the database key material is encrypted with a key derived from the auth password. Burner passwords are intended only for changing the auth password. Store the auth password, burners, and full database path in a password manager.

Use:

```bash
auth --change-password --dir ~/.auth
```

The recovery metadata includes an advisory machine identifier. If the database is copied to a different machine, the auth password may be required to restore the keys into that machine’s credential store.

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

| Platform     |       Status | Authorization backend                                           |
|--------------|-------------:|-----------------------------------------------------------------|
| macOS Tahoe  |  test target | Touch ID / password fallback through LocalAuthentication helper |
| Windows 11   |  test target | Windows Hello through `UserConsentVerifier`                     |
| Ubuntu 24.04 |  test target | PAM through `sudo -v` fallback                                  |
| Other Linux  | experimental | PAM through `sudo -v` fallback                                  |

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
AUTH_OPTIONS="-d ./auth-test" AUTH_TEST_FALLBACK_PASSWORD="Long-Test-Password-2026!" AUTH_TEST_FALLBACK_PASSWORD_CONFIRM="Long-Test-Password-2026!" AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="Long-Test-Password-2026!" auth --write important-script.sh
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

## Zsh completions

Zsh completion functions are provided in `completions/zsh/`.

To enable them for the current user:

```zsh
fpath=("$PWD/completions/zsh" $fpath)
autoload -Uz compinit
compinit
```

This enables completions for `auth`.

## Bash completions

Bash completion scripts are provided in `completions/bash/`.

To enable them for the current shell:

```bash
source "$PWD/completions/bash/auth"
```

## PowerShell completions

PowerShell completers are provided in `completions/powershell/`.

To enable them for the current session:

```powershell
. "$PWD/completions/powershell/auth.ps1"
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

For development and CI, use an `auth-test` database plus the test-only `AUTH_TEST_*` password environment variables.

## Integration tests

Run:

```bash
cargo test --all-targets --all-features
```

The CLI integration tests cover help/version output, writing authorization for two files, checking authorized/unauthorized/missing files, removing one authorization record, and detecting content changes.


## Test-only authorization bypass

`--no-platform-auth` has been removed from the CLI. Tests should use an `auth-test` database and the test-only `AUTH_TEST_*` password environment variables instead.

Example:

```bash
echo "Hello World" > TESTFILE.txt
mkdir -p auth-test
export AUTH_OPTIONS="-d ./auth-test"
export AUTH_TEST_FALLBACK_PASSWORD="Long-Test-Password-2026!"
export AUTH_TEST_FALLBACK_PASSWORD_CONFIRM="Long-Test-Password-2026!"
export AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="Long-Test-Password-2026!"
auth --write TESTFILE.txt
```

These `AUTH_TEST_*` variables are honored only for databases whose basename is exactly `auth-test`.

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

The `--no-platform-auth` option has been removed from the CLI; fallback-password recovery replaces that bypass.


### Authorization cache

`--cache-time=SECONDS` caches a successful platform/fallback authorization for the selected database for up to 120 seconds. The default is `0`, which disables caching. The cache entry is MAC-protected and tied to the current machine hash and database namespace.


## v0.8.4 CLI additions

Options in help are alphabetized. `--cache-time` now requires equals syntax, for example `--cache-time=60`, and remains limited to 120 seconds.

`--request-password` forces the Auth password / burner route instead of platform authorization. This is useful for CI, WSL, and systems where the platform prompt is unavailable. It is not a bypass; a valid Auth password or unused burner password is still required.

`--show-dir` displays the absolute auth directory and database path after authorization.

`--stats` displays the number of authorized file entries, the most recent successful write, and the most recent successful check after authorization.

### Root-relative path identity

Use `--root-dir=PATH` when a set of files should be authorized by their path relative to a canonical root directory rather than by their full absolute path. This is useful for packaged or copied trees where the same relative file layout may live under different parent directories.

```bash
auth --request-password --root-dir=/path/to/tree --write /path/to/tree/bin/tool
auth --root-dir=/other/copy/of/tree --check /other/copy/of/tree/bin/tool
```

Use `--root-dir=` to return to full-path identity when supplying options through `AUTH_OPTIONS`.
