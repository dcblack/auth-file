---
agent: "ChatGPT 5.5"
created: "2026-05-15T17:35:49+00:00"
version: "0.8.5"
---

# auth v0.8.5 update notes

## Implemented

- Added `--root-dir=PATH`.
- Added `AuthOptions.root_dir: Option<PathBuf>`.
- Added root-relative file identity for write, check, and remove.
- Kept full canonical-path identity as the default when no root is specified.
- Added `--root-dir=` behavior to reset to full-path identity when using `AUTH_OPTIONS`.
- Added unit and CLI tests for rooted portability.
- Bumped crate version to `0.8.5`.
- Bumped SQLite schema version to `6`.
- Kept `--stats` as the public option and used `auth_statistics` as the local variable to satisfy Clippy.

## Security model note

File authorization HMACs do not include host identity. Rooted path identity is therefore portable across machines when the same relative tree layout and database/key material are available.

Machine information is still used for recovery/cache metadata. Because it is not part of file identity, `--no-machine-lock` was not added in this draft.

## Root identity behavior

Default:

```bash
auth --write /absolute/path/to/file.txt
```

uses the canonical full path.

Rooted:

```bash
auth --root-dir=/absolute/path/to/tree --write /absolute/path/to/tree/pkg/file.txt
```

uses an HMAC over the relative identity:

```text
root:pkg/file.txt
```

This allows:

```bash
auth --root-dir=/other/location/tree --check /other/location/tree/pkg/file.txt
```

when the file content and relative path match.

## Validation to run

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## References

- https://doc.rust-lang.org/std/path/struct.Path.html
- https://doc.rust-lang.org/std/path/struct.PathBuf.html
- https://www.sqlite.org/pragma.html#pragma_user_version

## Original queries

- Add `--root-dir=PATH`.
- Support root-relative authorization identity.
- Allow portability of authorized files under different roots.
- Consider whether `--no-machine-lock` is needed.
