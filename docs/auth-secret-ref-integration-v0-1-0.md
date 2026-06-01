---
agent: "ChatGPT 5.5"
created: "2026-05-31T12:04:02+00:00"
version: "0.1.0"
---

# auth-file 1Password / Bitwarden secret-ref integration notes

## Recommended CLI additions

Add:

```text
--secret-ref=VALUE
```

Examples:

```bash
auth --secret-provider=1p      --secret-ref="op://Private/auth-file/password"
```

```bash
auth --secret-provider=bw      --secret-ref="bw://vault/item/password"
```

## Recommended parser structure

```rust
#[derive(Debug, Clone)]
pub struct SecretSource {
    pub provider: SecretProvider,
    pub secret_ref: String,
}
```

## Provider normalization

```rust
fn parse_secret_provider(value: &str) -> Result<SecretProvider, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1password" | "1p" | "1pw" => Ok(SecretProvider::OnePassword),
        "bitwarden" | "bw" => Ok(SecretProvider::Bitwarden),
        "env" | "environment" => Ok(SecretProvider::Environment),
        "keyring" | "keys" => Ok(SecretProvider::Keyring),
        other => Err(format!("unknown secret provider: {other}")),
    }
}
```

## 1Password command

```rust
Command::new("op")
    .arg("read")
    .arg(secret_ref)
```

## Bitwarden command

Likely:

```rust
Command::new("bw")
    .args(["get", "password", secret_ref])
```

or later support full JSON item extraction.

## Validation

Require:

```text
--secret-provider
--secret-ref
```

together.

## Help examples

```text
--secret-provider=1p
--secret-ref="op://Private/auth-file/password"
```

## Notes

Do NOT attempt to separately parse:
- vault
- item
- field

The native provider reference formats are already designed for this.

## References

- https://developer.1password.com/docs/cli/secret-references/
- https://developer.1password.com/docs/cli/reference/commands/read/
- https://bitwarden.com/help/cli/

## Original queries

- Add secret-ref based 1Password / Bitwarden integration.
