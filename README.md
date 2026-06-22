# auth-file / `auth`

`auth` is a small cross-platfom (macOS, Linux, Windows 11) command-line tool and Rust library for authorizing and validating files before scripts or automation use them.

The published crate name is `auth-file`; the installed binary is `auth`.

## Primary use case

The original motivating use case is safely sourcing a local setup file only when it still matches a previously authorized version:

```bash
cd "$directory" || exit 1

if auth --check setup.profile; then
    # shellcheck disable=SC1091
    source setup.profile
else
    echo "setup.profile is not authorized or has changed" >&2
    return 1 2>/dev/null || exit 1
fi
```

`--check` is intentionally read-only and does not require platform authorization, an Auth password, or a cached authorization session. It only verifies existing authorization records.

## Security model

`auth` separates two ideas:

1. **Authorization**: a trusted user approves a database-changing action such as `--write`, `--remove`, `--change-password`, `--show-dir`, or `--stats`.
2. **Validation**: later checks verify that a file still matches its authorized state.

Authorization records are stored in SQLite at `~/.auth/auth.db` by default. File path identities are stored as HMAC-SHA256 values, not plaintext paths and not plain path hashes. Content is stored as a SHA-256 digest and each record is protected with an Ed25519 signature.

Normal-use key material is stored through a secret provider. The default provider is the platform keyring provider. Databases whose directory basename is exactly `auth-test` use file-backed keys and test password environment variables so CI and development tests remain non-interactive.

Important limitations:

- `auth` protects against using changed files after authorization.
- It does not fully protect against malware already running as the same user.
- A valid authorization cache can be used by any command run as the same user during the cache window.
- SQLite is not encrypted; path privacy relies on keyed HMACs and protected local key material.

## Storage layout

Default per-user layout:

```text
~/.auth/
  auth.db
  auth-burners.age
  ed25519.verifying-key
```

Normal private key material is stored outside the database by the selected secret provider. `auth-burners.age` contains one-time burner passwords encrypted with the Auth password using the age file format.

On Unix-like systems the auth directory is set to `0700`. Private test key files in `auth-test` databases are set to `0600`.

## Auth password and burner passwords

Normal databases can establish an Auth password and ten one-time burner passwords. The Auth password is stored as an Argon2id password hash. A backup copy of database key material is encrypted with a key derived from the Auth password.

Burner passwords can authorize recovery operations exactly once. To avoid terminal scrollback exposure, generated burner passwords are written to an age-encrypted file instead of being printed directly:

```text
~/.auth/auth-burners.age
```

Decrypt it with `rage`, then save the burner passwords somewhere durable, preferably a password manager:

```bash
cargo install rage
rage -d ~/.auth/auth-burners.age > auth-burners.txt
```

If you forget the Auth password, this encrypted burner file will not help because it is protected by that same Auth password.

## Supported platforms

| Platform | Status | Authorization backend |
|---|---:|---|
| macOS | tested target | Touch ID / password fallback through LocalAuthentication helper |
| Windows 11 | tested target | Windows Hello through `UserConsentVerifier` when available |
| Ubuntu / WSL2 | tested target | Auth password fallback is the most reliable path |
| Other Linux | experimental | Auth password fallback; platform-specific biometric/PAM support is future work |

See `docs/platform-support.md` for more detail.

## CLI overview

```bash
auth --help
auth --version
auth --write  [OPTIONS] FILENAME...
auth --check  [OPTIONS] FILENAME...
auth --remove [OPTIONS] FILENAME...
auth --change-password [OPTIONS]
auth --show-config [OPTIONS]
auth --show-dir [OPTIONS]
auth --stats [OPTIONS]
```

Examples:

```bash
auth --write important-script.sh
auth --check important-script.sh
auth --remove important-script.sh
```

`--check` does not require authorization. Protected commands such as `--write`, `--remove`, `--change-password`, `--show-config`, `--show-dir`, and `--stats` require platform authorization, an Auth password, or a valid authorization cache depending on the selected options and platform.

## Configuration

`auth` reads configuration in this order:

1. TOML config file
2. `AUTH_OPTIONS`
3. command-line arguments

Later layers are appended after earlier layers, but duplicate-sensitive options such as root directives and secret provider selections are still enforced. For example, specifying both `--default-root` and `--root-dir=PATH` anywhere across config, `AUTH_OPTIONS`, and command line fails.

By default, `auth` looks for:

```text
~/.auth.toml
```

Use a different config file:

```bash
auth --config=/path/to/auth.toml --check file.txt
```

Disable config loading for one command:

```bash
auth --config= --write ~/.auth.toml
```

That is useful when authorizing or repairing the config file itself. `AUTH_CONFIG_DISABLE=1` is also still supported for tests and emergency troubleshooting, but `--config=` is the preferred user-facing spelling.
 
Display the selected configuration file and effective settings after authorization:

```bash
auth --request-password --show-config
```

Because effective configuration can reveal database locations and secret-provider references, `--show-config` is a protected command.

### TOML config format

Example:

```toml
# ~/.auth.toml
version = 1

default_root = true
cache_time = 60
color = "always"

secret_provider = "1password"
secret_ref = "op://VAULT/ITEM/FIELD"

# Escape hatch for options without first-class TOML keys:
options = ["--verbose"]
```

Supported structured keys:

| Key | Type | Meaning |
|---|---|---|
| `version` | integer | Configuration format version; currently must be `1` |
| `options` or `AUTH_OPTIONS` | string or array of strings | Extra auth options parsed before environment and command-line options |
| `cache_time` | integer | Authorization cache duration, 0 to 120 seconds |
| `color` | string | `auto`, `always`, or `never` |
| `default_root` | boolean | Use normal full-path identity |
| `dir` or `db_dir` | string | Auth database directory |
| `force` | boolean | Reserved for future non-security confirmation prompts |
| `quiet` | boolean | Report failures only |
| `request_password` | boolean | Prefer Auth password / burner authorization |
| `root_dir` | string | Use root-relative file identity |
| `secret_provider` | string | `prompt`, `env`, `os-keyring`, `1password`, or `bitwarden` |
| `secret_ref` | string | Provider-specific secret reference, such as `op://VAULT/ITEM/FIELD` |
| `silent` | boolean | Suppress routine output |
| `verbose` | boolean or integer | Increase verbosity; negative integer selects silent |

Supported test/helper environment variables in the TOML file:

```toml
AUTH_TEST_FALLBACK_PASSWORD = "Long-Test-Password-2026!"
AUTH_TEST_FALLBACK_PASSWORD_CONFIRM = "Long-Test-Password-2026!"
AUTH_TEST_CURRENT_PASSWORD_OR_BURNER = "Long-Test-Password-2026!"
AUTH_MACOS_TOUCHID_HELPER = "/path/to/auth-macos-touchid"
```

The process environment can also supply `AUTH_OPTIONS`, `AUTH_CONFIG_DISABLE`, `NO_COLOR`, `NOCOLOR`, and `PAGER`.

The `AUTH_TEST_*` variables are honored only for databases whose basename is exactly `auth-test`.

## Root-relative file identity

By default, `auth` uses full canonical file paths when computing file identity. Use `--root-dir=PATH` when a set of files should be authorized relative to a canonical root directory instead:

```bash
auth --request-password --root-dir=/path/to/tree --write /path/to/tree/bin/tool
auth --root-dir=/other/copy/of/tree --check /other/copy/of/tree/bin/tool
```

Use `--default-root` to explicitly select default full-path identity.

At most one root directive may appear across the config file, `AUTH_OPTIONS`, and command-line arguments. Root directives are:

```text
--default-root
--root-dir=PATH
```

A second root directive fails with:

```text
Error: Attempt to specify root directory more than once.
```

## Secret providers

Supported provider names and aliases:

| Provider | Aliases |
|---|---|
| `prompt` | |
| `env` | `environment` |
| `os-keyring` | `keyring`, `keys`, `oskeyring` |
| `1password` | `1p`, `1pw` |
| `bitwarden` | `bw` |

1Password support reads the configured secret reference with the 1Password CLI:

```bash
op read "op://VAULT/ITEM/FIELD"
```

Recommended configuration:

```toml
secret_provider = "1password"
secret_ref = "op://Private/auth-file/password"
```

Vault, item, and field names are user-defined in 1Password. `auth` does not try to infer them.

## Authorization cache

`--cache-time=SECONDS` caches a successful protected authorization for the selected database for up to 120 seconds. The default is `0`, meaning successful authorization does not create or refresh a cache entry.

A later protected command always checks for an existing unexpired cache first, even if that later command does not repeat `--cache-time`.

The cache entry is MAC-protected and tied to the database namespace and current machine hash. Tampered, expired, or machine-mismatched cache records are ignored and cleared.

## Color output

```bash
auth --color=auto   ...
auth --color=always ...
auth --color=never  ...
```

Errors are red, warnings are yellow, and passing/positive messages are green when color is enabled. `NO_COLOR` and `NOCOLOR` disable automatic color output. `--color=always` overrides those variables.

## Paged help

`auth --help` uses `$PAGER` when stdout is interactive. If `$PAGER` is unset, it tries `less -R`, then `more`, and finally falls back to plain stdout. In non-interactive contexts such as tests or pipes, help is printed directly.

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

On macOS, `build.rs` compiles `platform/macos/auth-macos-touchid.swift` into Cargo's `OUT_DIR` and embeds that helper path into the Rust binary. Runtime lookup order is:

1. `AUTH_MACOS_TOUCHID_HELPER`
2. helper compiled by `build.rs`
3. helper installed beside the `auth` executable
4. `auth-macos-touchid` found on `PATH`

## Tests

```bash
gmake verify
gmake tests-all
```

`tests.mk` contains randomized manual/system tests that should stand on their own after `tests-clear` and `tests-setup`.

## Security checks and SBOM

```bash
gmake audit
gmake sbom
```

Generated audit/SBOM output belongs under `artifacts/`.

## Packaging

```bash
cargo package
cargo publish --dry-run
```

See `docs/publishing-checklist.md` before publishing.

## License

Apache-2.0. See `LICENSE` and `NOTICE`.
