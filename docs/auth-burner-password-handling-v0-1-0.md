---
agent: "ChatGPT 5.5"
created: "2026-05-16T21:17:12+00:00"
version: "0.1.0"
---

# Burner password handling recommendation

## Recommendation

Do not print burner passwords to the terminal by default.

Instead, write them to an encrypted recovery file inside the auth directory, encrypted using the Auth password.

Suggested file:

```text
.auth/auth-burners.age
```

or, if using an internal format:

```text
.auth/auth-burners.enc
```

## Preferred approach

Use built-in Rust encryption rather than shelling out to GPG.

Recommended options:

1. Use the Rust `age` crate with passphrase encryption.
2. Or use the existing internal XChaCha20-Poly1305 + Argon2id code path already used for key backup.

## Why not require GPG?

GPG is strong and widely used, and GnuPG supports symmetric encryption with a passphrase. However, requiring GPG adds an external dependency and cross-platform setup friction.

For a self-contained Rust utility, built-in encryption is cleaner.

## UX proposal

After creating or changing the Auth password:

```text
Recovery burner passwords were written to:

  /Users/example/.auth/auth-burners.age

Decrypt this file and store the burner passwords somewhere safe, such as a password manager.
If you forget the Auth password, you may not be able to decrypt this file.
```

## Optional flags

Possible future switches:

```text
--print-burners
```

Print burner passwords to terminal only when explicitly requested.

```text
--burner-file=PATH
```

Write encrypted burner file to a specific location.

## Security notes

The encrypted burner file does not solve Auth password loss, because it is protected by that same Auth password. It mainly avoids terminal scrollback and logging exposure.

For true recovery if the Auth password is forgotten, the user must save the burner passwords somewhere independent of the Auth password.

## References

- https://github.com/FiloSottile/age
- https://docs.rs/age
- https://www.gnupg.org/gph/en/manual/r656.html
- https://www.gnupg.org/gph/en/manual.html
- https://docs.rs/rpassword/

## Original queries

- Avoid dumping burner passwords directly into the terminal.
- Consider encrypting burner passwords using the Auth password.
- Consider whether GPG is the right dependency or whether another approach is better.
