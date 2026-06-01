---
agent: "ChatGPT 5.5"
created: "2026-05-12T14:00:00-05:00"
version: "0.4.0"
title: "auth Rust build.rs and integration test update"
---

# auth Rust build.rs and integration test update

## Summary

Version 0.4.0 adds automated macOS Swift helper compilation through `build.rs` and expands CLI integration tests.

## build.rs behavior

On macOS, `build.rs` compiles:

```text
platform/macos/auth-macos-touchid.swift
```

into Cargo's `OUT_DIR` as:

```text
auth-macos-touchid
```

It then publishes that path to the Rust code using:

```text
cargo:rustc-env=AUTH_BUILT_MACOS_HELPER=...
```

On non-macOS platforms, the variable is set to an empty string.

## Runtime helper lookup order

The macOS authorization backend now searches for the helper in this order:

1. `AUTH_MACOS_TOUCHID_HELPER`
2. helper compiled by `build.rs`
3. helper installed beside the `auth` executable
4. `auth-macos-touchid` on `PATH`

## Integration tests added

The integration test suite now covers:

1. `--help`
2. `--version`
3. writing authorization for two files
4. checking two authorized files
5. checking one unauthorized file
6. checking one nonexistent file
7. checking a mixed list of authorized, unauthorized, and missing files
8. removing one authorization and then verifying the removed file fails
9. preserving the previous content-change detection test

All write/remove tests use:

```bash
--no-platform-auth
```

so the tests do not require Touch ID, Windows Hello, or PAM interaction.

## Commands to run

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

On macOS, `cargo build` should now compile the Swift helper automatically, assuming `swiftc` is available.

## Notes

If Swift is installed somewhere unusual, set:

```bash
export SWIFTC=/path/to/swiftc
```

If you want to override the helper at runtime, set:

```bash
export AUTH_MACOS_TOUCHID_HELPER=/path/to/auth-macos-touchid
```
