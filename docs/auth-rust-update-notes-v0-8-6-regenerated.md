---
agent: "ChatGPT 5.5"
created: "2026-05-16T21:48:28+00:00"
version: "0.8.6"
---

# auth v0.8.6 regenerated notes

This regenerated v0.8.6 package uses the uploaded fresh repo clone as the baseline so the local developer changes are preserved.

## Preserved from uploaded repo

- `GNUmakefile`
- `bin/` developer helper scripts
- updated `.gitignore`
- current v0.8.5 source tree and tests

## Added for v0.8.6

- Bumped crate version to `0.8.6`.
- Added `age` and `secrecy` dependencies.
- Replaced terminal dumping of burner passwords with an `age`-encrypted burner file.
- Burner file path: `auth-burners.age` inside the selected auth directory.
- Added README documentation pointing to `rage` on crates.io.
- Added CHANGELOG entry for v0.8.6.

## User workflow

Install `rage`:

```bash
cargo install rage
```

Decrypt the burner file:

```bash
rage -d ~/.auth/auth-burners.age > auth-burners.txt
```

Save the burner passwords in a password manager, then securely remove the plaintext output file.

## Validation

Run locally:

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

I could not compile-test this package in the sandbox because Cargo is not installed there.

## References

- https://crates.io/crates/rage
- https://crates.io/crates/age
- https://docs.rs/age

## Original queries

- Regenerate v0.8.6 using the uploaded repo snapshot as the baseline.
- Preserve the user’s Makefile, developer tooling, and `.gitignore` updates.
- Include the age/rage burner-file changes without creating collisions.
