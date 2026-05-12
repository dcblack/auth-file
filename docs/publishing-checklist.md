---
title: auth crates.io publishing checklist
version: 0.3.0
agent: ChatGPT 5.5
created: 2026-05-12T16:55:00-05:00
---

# crates.io publishing checklist

> Note: the crate name `auth` appears to already exist on crates.io. Rename the crate before publishing, for example `file-auth`, `script-auth`, `auth-file`, or an organization-prefixed name.

1. Replace placeholder `repository`, `homepage`, and `authors` in `Cargo.toml`.
2. Confirm the crate name is available.
3. Run formatting and lint checks.
4. Run unit and integration tests.
5. Generate and review the SBOM.
6. Run dependency audit.
7. Run `cargo package` and inspect the `.crate` contents.
8. Publish only after checking that no secrets, local test databases, or IDE files are included.

Commands:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo install cargo-audit cargo-cyclonedx
cargo audit
cargo cyclonedx --format json --output-file sbom.cdx.json
cargo package
cargo publish --dry-run
```
