# Contributing

## Local checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --test cli
cargo package --allow-dirty
```

## Security checks

```bash
cargo install cargo-audit cargo-cyclonedx
cargo audit
cargo cyclonedx --format json --output-file sbom.cdx.json
```

## Platform testing

Test at least:

- macOS Tahoe on Apple Silicon with Touch ID helper installed
- Windows 11 with Windows Hello configured
- Ubuntu 24.04 using PAM/sudo fallback

Use `--no-platform-auth` or `--force` only for CI and non-interactive testing.
