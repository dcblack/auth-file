---
agent: "ChatGPT 5.5"
created: "2026-05-16T00:00:00-05:00"
version: "0.8.6"
---

# auth v0.8.6 update notes

## Added

- Burner passwords are now written to an age-format encrypted file named `auth-burners.age`.
- The encrypted burner file is protected by the Auth password.
- Documentation points users to the `rage` CLI on crates.io: <https://crates.io/crates/rage>.

## Changed

- Burner passwords are no longer printed directly to the terminal by default.
- `--change-password` now reports the encrypted burner file path.

## User workflow

```bash
cargo install rage
rage -d ~/.auth/auth-burners.age > auth-burners.txt
```

Then move `auth-burners.txt` into a password manager and delete the plaintext file.

## Important limitation

If the user forgets the Auth password, they will not be able to decrypt `auth-burners.age`. The encrypted burner file prevents terminal leakage; it is not a substitute for saving the burner passwords somewhere independent.

## References

- https://crates.io/crates/rage
- https://crates.io/crates/age
- https://docs.rs/age
- https://github.com/str4d/rage

## Original queries

- Use age/rage instead of a custom encrypted burner-file format.
- Make the change part of v0.8.6.
- Document where to get `rage`, pointing to crates.io.
