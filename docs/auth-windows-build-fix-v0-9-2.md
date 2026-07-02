---
agent: "ChatGPT 5.5"
created: "2026-07-01T00:00:00"
version: "0.1.0"
---

# Windows build fix

Fixes Windows builds with `windows = 0.62` by replacing the removed blocking `.get()` call on `IAsyncOperation` with `futures_executor::block_on(op)`.

Changed files:

- `Cargo.toml`
- `src/lib.rs`

Validate on Windows with:

```powershell
cargo build --release --all-targets --all-features
cargo check --all-targets --all-features
cargo test --all-targets --all-features
```

## Original queries

- Package the Windows Hello async-operation build fix as changed files only.
