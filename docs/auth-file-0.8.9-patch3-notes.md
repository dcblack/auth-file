---
agent: "ChatGPT 5.5"
created: "2026-05-29T08:19:21+00:00"
version: "1.0.0"
---

# auth-file 0.8.9 patch3 notes

## Original queries

- Fix the hanging `secret_provider_parses` test.
- Preserve provider parsing coverage without invoking interactive or keyring-backed logic.

## Problem

`secret_provider_parses` used:

```text
auth --secret-provider=prompt --show-dir
```

That path can reach startup/auth initialization and may prompt, touch key storage, or otherwise block in a non-interactive test environment.

## Changes made

### `tests/cli.rs`

Changed `secret_provider_parses` to use:

```text
auth --secret-provider=prompt --version
```

This still exercises CLI parsing but exits before database/keyring/password initialization.

Added a 10-second timeout to the provider tests:

```rust
cmd.timeout(Duration::from_secs(10))
```

Changed duplicate-provider test to also use `--version` and a timeout:

```text
auth --secret-provider=prompt --secret-provider=env --version
```

This should fail during parsing/validation without ever reaching interactive code.

## Verification requested

Please run:

```bash
cargo fmt
cargo check
cargo test secret_provider_parses
cargo test duplicate_secret_provider_fails
cargo test
cargo clippy
```

## References

- https://docs.rs/assert_cmd/latest/assert_cmd/struct.Command.html
- https://doc.rust-lang.org/std/time/struct.Duration.html
