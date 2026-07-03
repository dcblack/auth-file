# Cargo.lock

The Rust community guideline is:

- **Applications / executables** → **Commit** **`Cargo.lock`**
- **Libraries published to crates.io** → **Do not commit** **`Cargo.lock`**

### **Why commit it?**

1. **Reproducible builds**
   - Every developer, CI system, and release builds against the exact same dependency graph.
2. **Security**
   - You know precisely which versions were tested.
   - SBOMs (`cargo cyclonedx`) and `cargo audit` become more meaningful because they refer to the locked dependency set.
3. **Cross-platform consistency**
   - If you validate on:
     - macOS
     - Linux
     - WSL
     - Windows
   - Everyone is using the same crate versions.
4. **Regression prevention**
   - Today’s Windows Hello issue is a perfect example. If a transitive dependency updates unexpectedly, you want to control *when* you adopt it, not discover it midway through a release.

### **My workflow recommendation**

For releases:

```bash
cargo update
cargo check
cargo clippy
cargo test
gmake tests-all
cargo audit
cargo cyclonedx
git commit Cargo.lock
```

Then tag and release.

### **When to update** **`Cargo.lock`**

Only intentionally.

For example:

```bash
cargo update
```

or

```bash
cargo update -p windows
```

Review the changes, run your full verification suite, then commit the updated lockfile.

### **One small addition**

Assuming you’re already building a disciplined release process, I’d add one line to your `CONTRIBUTING.md`:

`Cargo.lock` is committed for reproducible builds. Do not remove it. Update it only as part of an intentional dependency refresh.

------

I also have one suggestion related to your project layout. Once Windows is green, I’d like to add a `make deps-update` (or `make update-deps`) target that does something like:

```make
cargo update
cargo audit
cargo tree --duplicates
cargo cyclonedx --format=json --override-filename=artifacts/sbom/auth-file.cdx.json
```

It would give you a single, repeatable dependency maintenance command before each release. Given how much emphasis you’re placing on security, I think it would fit your workflow very well.