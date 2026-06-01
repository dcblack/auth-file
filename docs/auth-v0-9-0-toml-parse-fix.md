# v0.9.0 TOML parse fix

The failing config tests were caused by parsing the config file with:

```rust
let value: toml::Value = text.parse()?;
```

With the current `toml` crate, that attempts to parse a single TOML value, not a full TOML document/table. It fails on normal config files like:

```toml
options = ["--dir=/tmp/auth-test"]
```

This update changes the parser to:

```rust
let table: toml::Table = toml::from_str(&text)?;
```

and then iterates over `&table`.

Run:

```bash
gmake verify
```
