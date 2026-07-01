---
applyTo: **/*.rs
---

Prefer:
- Result<T,E>
- thiserror
- tracing

Avoid:
- unwrap()
- expect() except tests
