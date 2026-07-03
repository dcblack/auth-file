# auth 0.9.3

## Summary

This release hardens default state/config lookup and fixes native Windows compilation of the Windows Hello authorization path.

## Changes

- Default Unix/macOS home-directory lookup no longer trusts the `HOME` environment variable.
- Tests can isolate default config/home lookup with `AUTH_TEST_HOME`; this override is intended only for debug/test builds.
- Native Windows Hello authorization now uses `IAsyncOperation::join()` for the `windows` crate 0.62 API.
- CLI tests were updated so default config tests no longer require changing a developer's real home directory.

## Security rationale

A malicious script can set `HOME` before invoking `auth`. If default config and database paths trusted that environment variable, the script could redirect `auth` toward attacker-controlled state. `auth` now derives Unix/macOS default paths from OS account data instead of `HOME`, while keeping an explicit test-only override for isolated tests.

## Windows note

The Windows default home/profile lookup is still centralized for future hardening. Native Windows validation should be run from PowerShell using `cargo check`, `cargo clippy`, and `cargo test`; Git Bash/WSL builds may report a Linux target and do not exercise Windows Hello.

## Validation

Recommended checks:

```sh
gmake verify
gmake tests-all
```

On native Windows PowerShell:

```powershell
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Original queries

- Implement 0.9.3 with Windows test fixes and home security hardening.
