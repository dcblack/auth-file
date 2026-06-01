---
agent: "ChatGPT 5.5"
created: "2026-05-30"
version: "0.1.0"
---

# tests.mk standalone fix

## Change

`test-auth-options` no longer relies on `test-root-dir` having run first.

The target now creates its own isolated:

```text
/tmp/auth-file-manual-tests/auth-options/auth-test
/tmp/auth-file-manual-tests/auth-options/root
```

It writes and authorizes its own rooted file, then checks it using `AUTH_OPTIONS`.

## Reason

`make tests-all` now randomizes test order, so every `test-*` target must be independent after `tests-clear` and `tests-setup`.

## Original queries

- Diagnose failed randomized `test-auth-options`.
- Make the target stand alone.
