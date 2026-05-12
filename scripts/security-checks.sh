#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

if command -v cargo-audit >/dev/null 2>&1; then
  cargo audit
else
  echo "warning: cargo-audit not installed; skipping dependency vulnerability audit" >&2
fi

if command -v cargo-cyclonedx >/dev/null 2>&1; then
  cargo cyclonedx --format json --output-file sbom.cdx.json
else
  echo "warning: cargo-cyclonedx not installed; skipping SBOM generation" >&2
fi

if command -v cargo-deny >/dev/null 2>&1; then
  cargo deny check advisories bans licenses sources
else
  echo "warning: cargo-deny not installed; skipping cargo-deny policy check" >&2
fi
