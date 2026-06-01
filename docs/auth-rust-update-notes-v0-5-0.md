---
agent: "ChatGPT 5.5"
created: "2026-05-12T21:58:00-05:00"
version: "0.5.0"
---

# auth Rust update notes v0.5.0

## Summary

This update tightens the `--no-platform-auth` development bypass, adds `AUTH_OPTIONS`, adds colorized status output, and adds paged help behavior.

## Security change: restricted `--no-platform-auth`

`--no-platform-auth` is now allowed only when both conditions are true:

1. `--dir` or `-d` is explicitly supplied.
2. The database directory basename is exactly `auth-test`.

This means the following works for development and CI:

```bash
echo "Hello World" > TESTFILE.txt
mkdir -p auth-test
export AUTH_OPTIONS="-d ./auth-test"
auth --no-platform-auth --write TESTFILE.txt
```

But these fail:

```bash
auth --no-platform-auth --write TESTFILE.txt
auth --no-platform-auth -d ~/.auth --write TESTFILE.txt
auth --no-platform-auth -d ./db --write TESTFILE.txt
```

When `--no-platform-auth` is active, `auth` prints a warning unless `--silent` is used.

## AUTH_OPTIONS

`AUTH_OPTIONS` is parsed before the command line. This is intended for repeated options such as the test database directory:

```bash
export AUTH_OPTIONS="-d ./auth-test"
```

Command-line arguments are appended afterward.

## Color output

Added:

```bash
--color auto
--color always
--color never
```

Color meanings:

- errors: red
- warnings: yellow
- passing/positive messages: green

`NO_COLOR` and `NOCOLOR` disable automatic color output. `--color always` forces color.

## Paged help

`auth --help` now uses a pager when stdout is interactive:

1. `$PAGER`, if set
2. `less -R`
3. `more`
4. direct stdout fallback

In non-interactive contexts, help prints directly.

## Tests added

New or updated integration tests cover:

- `--no-platform-auth` requires an explicit `auth-test` directory
- `AUTH_OPTIONS` can supply the test directory
- warning output when `--no-platform-auth` is active
- forced color output for errors
- `NO_COLOR` behavior for automatic color output

## Validation note

This package was edited structurally, but Cargo is not available in this execution environment, so final compilation should be validated locally with:

```bash
cargo fmt --all
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```
