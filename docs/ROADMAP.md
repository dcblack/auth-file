# **auth Roadmap**

## **Current Status**

Current development release:

**0.9.3** (approaching completion)

Major accomplishments:

- ✅ Native macOS support
- ✅ Native Linux support
- ✅ Native Windows support (Windows Hello in progress)
- ✅ WSL support
- ✅ SQLite authorization database
- ✅ Ed25519 signatures
- ✅ Authorization cache
- ✅ Secret providers (not entirely verified)
  - Prompt
  - Environment
  - Keyring
  - 1Password
  - Bitwarden
- ✅ TOML configuration
- ✅ Effective configuration display
- ✅ Extensive automated tests
- ✅ Cross-platform build verification

------

# **0.9.4 — Windows Stabilization**

Status: In Progress

## **Goals**

### **Windows**

- Fix remaining TOML test generation issues.
- Validate native Windows Hello.
- Ensure all CLI tests pass on:
  - macOS
  - Ubuntu
  - WSL2
  - Windows 11 PowerShell

### **Security**

- Stop trusting `HOME` / `USERPROFILE` for locating the default authorization store.
- Use OS-native user profile discovery.
- Introduce test-only home override.

### **Testing**

- Continue eliminating platform-specific assumptions.
- Keep tests completely isolated from user configuration.

------

# **0.9.5 — Release Engineering**

## **Build**

- Improve Windows build experience.
- Continue tool version verification.

Potential additions:

```
make tools-current
make tools-blessed
make tools-check
```

Integrate into:

```
make verify
```

------

## **Security**

Investigate:

- database integrity verification
- database signatures
- secure configuration loading

------

# **0.10.0 — Refactoring**

Status:

Planned major cleanup.

## **Goals**

Split `src/lib.rs`.

Proposed modules:

```
config.rs
cache.rs
crypto.rs
database.rs
authorization.rs
secret_provider.rs
portable_identity.rs
stats.rs
paths.rs
errors.rs
```

Objectives:

- smaller files
- clearer ownership
- easier testing
- improved comments

------

# **0.11.0 — Directory Authorization**

One of the largest planned features.

## **Objectives**

Authorize directories.

Directory identity should include:

- directory name
- metadata
- recursive manifest

Fast mode:

Hash:

- names
- sizes
- modification timestamps

Strict mode:

Hash complete file contents.

Future possibility:

```
auth --write directory/
auth --check directory/
```

------

# **Future**

## **Git Awareness**

Possibilities:

```
auth --git
```

Honor:

```
.gitignore
```

Possibly:

- authorize repository state
- detect unauthorized modifications

------

## **Security Enhancements**

Investigate:

Database signature.

Possible layout:

```
.auth/
    auth.db
    auth.db.sig
```

------

## **Secret Providers**

Continue expanding.

Possible additions:

- Apple Passwords
- KeePassXC
- HashiCorp Vault
- Azure Key Vault
- AWS Secrets Manager

------

## **Configuration**

Continue improving TOML.

Possible future:

```toml
[defaults]

[secret]

[database]

[display]
```

instead of everything in one flat namespace.

------

## **Testing**

Long-term goal:

Every release passes:

- macOS
- Ubuntu
- WSL2
- Windows

using exactly the same release process.

------

# **Documentation**

Continue improving:

```
README.md
ARCHITECTURE.md
ROADMAP.md
SECURITY.md
CONTRIBUTING.md
```

Also:

```
docs/releases/
```

for design rationale.

------

# **Guiding Principles**

- Security before convenience.
- Explicit is better than implicit.
- Cross-platform behavior should be identical whenever practical.
- Deterministic, reproducible builds.
- Extensive automated testing before refactoring.
- Keep the CLI simple and unsurprising.
- Document design decisions, not just code.

------

## **A few additions I’d make**

There are a few ideas that have come up repeatedly and deserve to be explicit roadmap items:

### **Release 1.0 Checklist**

Rather than letting “1.0” creep up on us, I’d create a section listing the criteria for declaring the project production-ready, such as:

- Stable CLI (no planned breaking changes)
- 100% passing tests on all supported platforms
- Stable configuration format
- Complete user documentation
- Security review complete
- External beta feedback incorporated

### **Air-Gapped Environment Support**

Given your interest in classified and offline environments, I’d also add:

- Zero network dependency after installation
- Reproducible builds
- Versioned `Cargo.lock`
- Version-controlled SBOM (e.g., `security/sbom/`)
- Toolchain verification (`tools-versions.txt`)
- Documented offline installation process

I think those goals fit naturally with the project’s emphasis on security and reproducibility, and they’ll help distinguish `auth` as a tool that’s suitable for high-assurance environments rather than just a convenient file authorization utility.