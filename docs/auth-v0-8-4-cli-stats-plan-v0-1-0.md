---
agent: "ChatGPT 5.5"
created: "2026-05-14T21:01:36+00:00"
version: "0.1.0"
---

# auth v0.8.4 planning notes

## Requested additions

1. Sort help/documentation options alphabetically.
2. Add `--show-dir`.
3. Require `--cache-time=SECONDS` syntax.
4. Add `--stats`.

## Decisions

### 1. Alphabetized help

Alphabetize long options in help output. Short aliases may remain attached to their long option.

Recommended order:

```text
--cache-time=SECONDS
--change-password
--check, -ck
--color MODE
--dir, -d DIR
--force, -f
--help, -h
--quiet, -q
--remove, -rm
--request-password
--show-dir
--silent, -s
--stats
--verbose, -v
--version
--write, -wr
```

### 2. `--show-dir`

`--show-dir` should display the absolute database directory path.

Because the path reveals where sensitive authorization material lives, it should require authorization.

Recommended output:

```text
Auth directory: /absolute/path/to/.auth
Database: /absolute/path/to/.auth/auth.db
```

### 3. `--cache-time=SECONDS`

Require equals syntax only.

Accepted:

```bash
auth --cache-time=60 -wr file.txt
```

Rejected:

```bash
auth --cache-time 60 -wr file.txt
```

Limit remains 0-120.

### 4. `--stats`

`--stats` should require authorization.

Suggested output:

```text
Auth directory: /absolute/path/to/.auth
Database: /absolute/path/to/.auth/auth.db
Authorized file entries: 12
Most recent write: 2026-05-14T14:22:11Z
Most recent check: 2026-05-14T15:03:09Z
```

## Schema change

Bump schema version.

Add metadata table:

```sql
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

Update metadata:

- On successful write:
  - `last_write_unix = unix_now()`
- On successful check:
  - `last_check_unix = unix_now()`

Stats query:

```sql
SELECT COUNT(*) FROM records;
SELECT value FROM metadata WHERE key = 'last_write_unix';
SELECT value FROM metadata WHERE key = 'last_check_unix';
```

## CLI state additions

Avoid more booleans by introducing:

```rust
enum CommandMode {
    FileActions,
    ChangePassword,
    ShowDir,
    Stats,
}
```

This also avoids Clippy `struct_excessive_bools`.

## Authorization behavior

`--show-dir`, `--stats`, `--write`, `--remove`, and `--change-password` require authorization.

`--check` does not require authorization, but it should update `last_check_unix` on successful checks.

## Recommended implementation order

1. Add `CommandMode`.
2. Add `--request-password`.
3. Add `--cache-time=SECONDS`.
4. Add `--show-dir`.
5. Add `metadata` table.
6. Add `--stats`.
7. Alphabetize help text.
8. Add tests.

## Original queries

- Sort help/documentation options alphabetically.
- Add `--show-dir`.
- Require `--cache-time=SECONDS`.
- Add `--stats`.
