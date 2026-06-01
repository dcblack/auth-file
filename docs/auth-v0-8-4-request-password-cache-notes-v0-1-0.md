---
agent: "ChatGPT 5.5"
created: "2026-05-14T20:14:21+00:00"
version: "0.1.0"
---

# auth v0.8.4 proposed changes: `--request-password` and cache testing

## Summary

Add a `--request-password` CLI option that forces the fallback Auth password / burner password route. This is useful for CI, WSL, Windows machines where Windows Hello app consent is unavailable, and repeatable testing.

Keep `--cache-time SECONDS` with a hard maximum of 120 seconds and default 0.

## Design decision

`--request-password` is not a bypass. It still requires a valid configured fallback password or one valid unused burner password.

## Recommended CLI behavior

```bash
auth --request-password -wr file.txt
auth --request-password --cache-time 60 -wr file1.txt -wr file2.txt
```

## Testing flow

First test without cache:

```bash
AUTH_OPTIONS="-d ./auth-test" \
AUTH_TEST_FALLBACK_PASSWORD="Long-Test-Password-2026!" \
AUTH_TEST_FALLBACK_PASSWORD_CONFIRM="Long-Test-Password-2026!" \
AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="Long-Test-Password-2026!" \
auth --request-password -wr TESTFILE1.txt
```

Then test with cache:

```bash
AUTH_OPTIONS="-d ./auth-test" \
AUTH_TEST_CURRENT_PASSWORD_OR_BURNER="Long-Test-Password-2026!" \
auth --request-password --cache-time 60 -wr TESTFILE2.txt -wr TESTFILE3.txt
```

Expected behavior: the first protected action authenticates with the password; the next protected action within 60 seconds uses the cache.

## Implementation direction

Add an authorization preference enum or mode:

```rust
pub enum AuthorizationMode {
    Platform,
    Password,
    None, // optional internal-only/test-only if still used by unit tests
}
```

Then in CLI parsing:

```rust
"--request-password" => {
    state.options.authorization = AuthorizationMode::Password;
}
```

The protected-action authorization flow should become:

1. If cache is valid, accept.
2. Else if mode is `Password`, call `authenticate_with_fallback_or_burner`.
3. Else if mode is `Platform`, try platform auth, and if that fails, warn and call `authenticate_with_fallback_or_burner`.
4. On success, update cache if `cache_seconds > 0`.

## Original queries

- Use cached authorization during testing with 60 seconds.
- Add `--request-password` to force the fallback password route, especially for CI.
