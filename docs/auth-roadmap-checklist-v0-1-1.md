---
agent: "ChatGPT 5.5"
created: "2026-05-18T20:38:51+00:00"
version: "0.1.0"
---

# auth-file roadmap checklist

## Notes

- `rage` burner-file generation appears implemented already.
- `--change-password` DOES appear in the current help text.
- This checklist focuses on discussed-but-not-fully-implemented items.
- I've added the intended version below
- I also added some Q: questions and other comments enclosed in parentheses 

---

# 0.8.7 Root handling

- [x] Implement `--default-root`
- [x] Enforce `--root-dir` xor `--default-root`
- [x] Detect duplicate root directives across `AUTH_OPTIONS`
- [x] Emit exact error:
      `Error: Attempt to specify root directory more than once.`
- [x] Add root-directive tests
- [x] Reject old split syntax:
      `--root-dir PATH`
- [x] Require:
      `--root-dir=PATH`

---

# 0.8.8 Authorization / security model

- [x] Make normal `--check` fully read-only
- [ ] Stop updating `last_check` during normal `--check`(Q: Won't this prevent sleuthing issues?)
- [ ] Add optional `--record-access` (Q: Is't this backwards? See above.)
- [x] Add read-only SQLite verification mode
- [ ] Add machine rebind flow (Q: Remind me what this is)
- [x] Improve stable machine identity
- [x] Implement `--portable`
- [x] Add secure machine-lock migration workflow

---

# 0.8.7 Burner / recovery workflow

- [x] Stop printing burner passwords by default (Q: I think this is done)
- [x] Generate encrypted `auth-burners.age` (Q: I think this is done)
- [ ] Add `--print-burners` (Q: How would this work? PDF? Direct to printer?)
- [x] Add burner regeneration workflow
- [x] Add burner revocation workflow
- [x] Add burner expiration policy
- [x] Add burner remaining-count reporting
- [ ] Formalize/version recovery blob format (Q: Remind me what this is)

---

# 0.8.9 Shared / team verification

- [ ] Add exported verification DB
- [x] Add:
      `auth --export-verify-db` (Q: How does the other side import this?)
- [x] Add:
      `auth --verify-db=PATH`
- [x] Separate:
      `private/auth.db`
      `shared/verify.db`
- [x] Add Unix group-sharing model
- [x] Add read-only verifier role
- [x] Document safe shared usage

---

# 0.8.7 Windows support

- [x] Implement proper Windows ACL handling
- [x] Add admin/checker ACL roles
- [x] Improve Windows Hello fallback handling
- [x] Evaluate DPAPI integration

---

# 0.8.7 Linux support

- [x] Replace `sudo -v` fallback with direct PAM support
- [x] Investigate `polkit`
- [x] Investigate `fprintd`
- [x] Investigate biometric PAM integration

---

# 0.8.10 Documentation

- [x] Document safe:
      `auth --check setup.profile && source setup.profile` (Q: Should add --default-profile to that auth)
- [x] Add team/shared verification examples
- [x] Add portable-project examples
- [x] Add machine-locking explanation
- [x] Add recovery/burner guidance
- [x] Audit help text for missing options
- [x] Ensure help options remain alphabetically sorted

---

# 0.8.11 CI / supply chain

- [x] Add GitHub Actions CI
- [x] Add macOS CI
- [x] Add Windows CI
- [x] Add Ubuntu CI
- [x] Add `cargo deny`
- [x] Resolve/transitively upgrade `keyring` audit warnings
- [x] Add release artifact automation
- [x] Add checksum/signature generation

---

# Testing (as features are added)

- [x] Expand `tests.mk`
- [x] Add root-directive conflict tests
- [x] Add portable-root tests
- [x] Add machine-lock tests
- [x] Add shared-verification tests
- [x] Add burner-regeneration tests
- [x] Add Windows-specific tests
- [x] Add Linux PAM tests

---

# Possible future ideas

- [x] Detached signed manifests
- [x] Recursive directory authorization (Q: What happens today when we authorize a directory?)
- [ ] Trust policies
- [ ] Multi-user signing
- [ ] File quarantine integration
- [x] Git integration hooks
- [ ] Project trust profiles

---

## References

- https://crates.io/crates/rage
- https://crates.io/crates/age
- https://rustsec.org/
- https://www.sqlite.org/
- https://learn.microsoft.com/en-us/windows/win32/secauthz/access-control-lists

## Original queries

- Assemble discussed-but-not-yet-implemented items into a markdown checklist.
- Verify whether `--change-password` appears in help.
