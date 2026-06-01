---
agent: "ChatGPT 5.5"
created: "2026-05-18T09:02:58+00:00"
version: "0.1.0"
---

# auth root option hardening notes

## Requested change

Add:

```text
--default-root
```

Meaning: use the default full-path identity behavior.

Existing:

```text
--root-dir=PATH
```

Meaning: use root-relative file identity.

## Security rule

Only one root directive may appear across the fully expanded command, including `AUTH_OPTIONS`.

Root directives are:

```text
--root-dir=PATH
--default-root
```

Any second root directive should fail with exactly:

```text
Error: Attempt to specify root directory more than once.
```

## Important interpretation

The test list implies the strict rule is safer:

- `AUTH_OPTIONS="--root-dir=..." auth --root-dir=...` => error
- `AUTH_OPTIONS="--default-root" auth --root-dir=...` => error
- `AUTH_OPTIONS="--root-dir=..." auth --default-root` => error

So this does not silently allow command-line root override. It rejects ambiguity/subterfuge.

## Suggested data model

Simple option-state tracking is enough:

```rust
struct CliState {
    root_directive_seen: bool,
    // existing fields...
}
```

In `AuthOptions`:

```rust
pub root_dir: Option<PathBuf>,
```

Where:

- `root_dir == None` means default full path
- `--default-root` sets `root_dir = None`
- `--root-dir=PATH` sets `root_dir = Some(PathBuf::from(PATH))`

## Suggested helper

```rust
const ROOT_SPECIFIED_MORE_THAN_ONCE: &str =
    "Attempt to specify root directory more than once.";

fn note_root_directive(state: &mut CliState) -> Result<(), String> {
    if state.root_directive_seen {
        return Err(ROOT_SPECIFIED_MORE_THAN_ONCE.to_string());
    }
    state.root_directive_seen = true;
    Ok(())
}
```

## Parser changes

For `--root-dir=PATH`:

```rust
value if value.starts_with("--root-dir=") => {
    note_root_directive(state)?;
    let root = value
        .strip_prefix("--root-dir=")
        .ok_or_else(|| "invalid --root-dir syntax".to_string())?;

    if root.is_empty() {
        state.options.root_dir = None;
    } else {
        state.options.root_dir = Some(PathBuf::from(root));
    }
}
```

For `--default-root`:

```rust
"--default-root" => {
    note_root_directive(state)?;
    state.options.root_dir = None;
}
```

Reject old split syntax:

```rust
"--root-dir" => {
    return Err("use --root-dir=PATH".to_string());
}
```

## AUTH_OPTIONS handling

The cleanest implementation is:

1. Expand `AUTH_OPTIONS` into an argument vector.
2. Append real command-line arguments.
3. Parse once.
4. Let `root_directive_seen` detect any second root directive.

That automatically satisfies:

- once in command line passes
- once in `AUTH_OPTIONS` passes
- once in `AUTH_OPTIONS` + once in command line fails
- two in command line fails
- no specification implies default root

## Tests to add

### 1. `--default-root` once passes

```bash
auth --default-root --check file
```

### 2. `--root-dir=PATH` once passes

```bash
auth --root-dir=/tmp/root --check /tmp/root/file
```

### 3. `--root-dir` more than once fails

```bash
auth --root-dir=/tmp/a --root-dir=/tmp/b --check file
```

Expected stderr:

```text
Error: Attempt to specify root directory more than once.
```

### 4. `--default-root` and `--root-dir` in command fails

```bash
auth --default-root --root-dir=/tmp/root --check file
```

Expected same error.

### 5. `AUTH_OPTIONS` with either root directive once passes

```bash
AUTH_OPTIONS="--default-root" auth --check file
AUTH_OPTIONS="--root-dir=/tmp/root" auth --check /tmp/root/file
```

### 6. `AUTH_OPTIONS` root combined with command-line root fails

```bash
AUTH_OPTIONS="--root-dir=/tmp/a" auth --root-dir=/tmp/b --check file
AUTH_OPTIONS="--default-root" auth --root-dir=/tmp/b --check file
AUTH_OPTIONS="--root-dir=/tmp/a" auth --default-root --check file
```

### 7. No root specification implies default root

```bash
auth --check file
```

## Documentation/help addition

Alphabetically, add:

```text
--default-root              Use default full-path file identity
```

Keep:

```text
--root-dir=PATH             Use PATH as root for relative file identity
```

## Original queries

- Add `--default-root`.
- Enforce root specification only once.
- Add tests for command-line, `AUTH_OPTIONS`, duplicates, and default behavior.
