# v0.9.0 effective config + shadow-rs notes

Changed files generated from uploaded archive `auth-file-1780280785.zip`.

## Includes

- Expanded TOML config parsing for structured keys:
  - `cache_time`
  - `color`
  - `default_root`
  - `dir` / `db_dir`
  - `force`
  - `quiet`
  - `request_password`
  - `root_dir`
  - `secret_provider`
  - `silent`
  - `verbose`
- Preserves `options` / `AUTH_OPTIONS` array or string support.
- Introduces an `EffectiveConfig` assembly stage for config/env/CLI layering.
- Adds `shadow-rs` build metadata wiring in `build.rs`.
- Updates `--version` to print build metadata.
- Adds tests for structured TOML options and new version output.

## Validate

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
gmake verify
```

## Note

I could not run Cargo in the sandbox.
