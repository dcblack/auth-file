---
agent: "ChatGPT 5.5"
created: "2026-05-29T09:47:29+00:00"
version: "1.0.0"
---

# auth-file 0.8.9 patch4 notes

## Original queries

- Fix `auth --check` asking for a password when it should not.
- Change `--version` from `auth #.#.#` to `auth #.#.# (release)` or `auth #.#.# (dev)` based on build profile / optimization profile.

## Changes made

### `src/lib.rs`

- Changed `AuthOptions::default().secret_provider` back to:

```rust
SecretProvider::OsKeyring
```

Rationale: `prompt` as the default secret-storage provider makes later `--check` operations require decrypting key material with the auth password. That violates the intended model where `--check` validates records without user authorization.

- Changed `auth_report()` so recovery/password initialization only occurs for sensitive operations:

```rust
ActionType::Write | ActionType::Remove
```

`ActionType::Check` still loads the database/key material needed to verify files, but does not initialize or request recovery/auth-password setup.

### `build.rs`

- Reads Cargo's `PROFILE`.
- Exports:

```text
AUTH_BUILD_KIND=release
```

when `PROFILE=release`, otherwise:

```text
AUTH_BUILD_KIND=dev
```

### `src/main.rs`

- Changed first-option `--version` output to:

```text
auth 0.8.9 (dev)
```

or:

```text
auth 0.8.9 (release)
```

### `tests/cli.rs`

- Updated `version_option_works` to expect `(dev)` during normal test builds.
- Reworked `secret_provider_parses` so it actually parses `--secret-provider=prompt` without relying on `--version` short-circuit behavior.
- Kept a timeout guard on provider-related tests.

### `golden/version.txt`

- Updated to:

```text
auth 0.8.9 (dev)
```

## Verification requested

Please run:

```bash
cargo fmt
cargo check
cargo test version_option_works
cargo test secret_provider_parses
cargo test
cargo clippy
```

Also manually verify release output:

```bash
cargo run --release -- --version
```

Expected:

```text
auth 0.8.9 (release)
```

## References

- https://doc.rust-lang.org/cargo/reference/environment-variables.html
- https://doc.rust-lang.org/cargo/reference/profiles.html
- https://doc.rust-lang.org/reference/conditional-compilation.html
