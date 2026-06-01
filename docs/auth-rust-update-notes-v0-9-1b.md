# v0.9.1+b test isolation fix

This changed-files package updates `tests/cli.rs`.

## Change

Two regression tests intentionally remove the Auth password test environment variables to prove that `--check` does not request authentication.

Those tests also need to disable default config loading. Otherwise a real `~/.authrc` on the developer machine can leak into the test process and cause unrelated config parsing/provider errors.

Updated tests:

- `check_with_cache_time_does_not_request_password`
- `check_with_request_password_does_not_request_password`

Both now set:

```text
AUTH_CONFIG_DISABLE=1
```

## Validation

```bash
gmake verify
```
