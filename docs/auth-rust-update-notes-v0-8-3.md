---
agent: "ChatGPT 5.5"
created: "2026-05-14T19:42:42+00:00"
version: "0.8.3"
---

# auth-rust v0.8.3 update notes

## Changes

1. Changed user-facing prompt text from “Fallback password:” to “Auth password:”.
2. Platform authorization fallback now prints a clearer warning before prompting for the Auth password.
3. Burner passwords are accepted by the normal write/remove authorization fallback path and are burned after a successful match.
4. Linux and WSL no longer use the incomplete `sudo -v` authorization backend; they fall through to Auth password fallback instead.
5. Added a unit test proving a burner password works exactly once.
6. Preserved v0.8.2 `--cache-time SECONDS` support.

## Validation requested

Run:

```bash
cargo fmt --all
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Then test manually on macOS, Windows, Ubuntu, and WSL.

## Known limitation

The normal Auth password can restore encrypted key backup material. Burner passwords currently authorize operations and password changes, but they do not decrypt the encrypted key backup by themselves. Supporting burner-based key restore would require storing a separate encrypted key backup per burner password.

## References

- https://docs.rs/rpassword
- https://doc.rust-lang.org/cargo/commands/cargo-test.html
- https://www.sqlite.org/pragma.html#pragma_user_version

## Original queries

- Improve fallback/Auth password behavior after testing.
- Make burner passwords usable exactly once where the normal Auth password is accepted.
- Rename prompts to “Auth password”.
- Fix WSL/Linux behavior when no platform prompt appears.
