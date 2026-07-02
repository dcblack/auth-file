# auth 0.9.2 Windows Hello native build fix

## Issue

Native Windows builds using `windows` 0.62 failed because the previous code called
`IAsyncOperation::get()`, which is no longer available on the async operation type
returned by `UserConsentVerifier::RequestVerificationAsync`.

## Fix

Use `IAsyncOperation::join()` from the `windows-future` async support crate that is
pulled in by the `windows` crate.

This keeps the Windows Hello authorization path synchronous from `auth`'s point of
view while matching the current Windows crate API.

## Validation

Run on native Windows PowerShell, not WSL/Git Bash:

```powershell
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

If `auth --version` reports `target: x86_64-unknown-linux-gnu`, that build is not
exercising the native Windows code path.
