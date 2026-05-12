#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

//! `authlib` - file authorization and validation library.
//!
//! The crate intentionally separates authorization from validation:
//! - `Write` and `Remove` may require user authorization.
//! - `Check` verifies stored cryptographic records without prompting.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const KEYPAIR_FILE: &str = "ed25519.signing-key";
const PATH_KEY_FILE: &str = "path-hmac.key";
const PUBKEY_FILE: &str = "ed25519.verifying-key";
const RECORD_EXT: &str = "auth.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Check,
    Write,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationMode {
    /// Do not ask for biometric/PAM/Hello approval. Useful for CI or bootstrap.
    None,
    /// Use the best available platform prompt. Falls back to a denial if unavailable.
    Platform,
}

#[derive(Debug, Clone)]
pub struct AuthOptions {
    pub db_dir: PathBuf,
    pub verbose: i8,
    pub force: bool,
    pub authorization: AuthorizationMode,
    pub reason: String,
}

impl Default for AuthOptions {
    fn default() -> Self {
        Self {
            db_dir: default_db_dir(),
            verbose: 0,
            force: false,
            authorization: AuthorizationMode::Platform,
            reason: "Authorize file trust database change".to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AuthReport {
    pub checked: usize,
    pub written: usize,
    pub removed: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

impl AuthReport {
    pub fn ok(&self) -> bool {
        self.failed == 0
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("invalid signing key")]
    InvalidSigningKey,
    #[error("invalid verifying key")]
    InvalidVerifyingKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("authorization denied: {0}")]
    AuthorizationDenied(String),
    #[error("unsupported platform authorization: {0}")]
    UnsupportedAuthorization(String),
    #[error("file is not readable: {0}")]
    FileNotReadable(PathBuf),
    #[error("authorization record does not exist for {0}")]
    RecordMissing(PathBuf),
    #[error("validation failed for {0}: {1}")]
    ValidationFailed(PathBuf, String),
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthRecord {
    version: u32,
    tool: String,
    created_unix: u64,
    path_hmac_sha256: String,
    content_sha256: String,
    size: u64,
    signature: String,
}

#[derive(Debug, Serialize)]
struct SignedPayload<'a> {
    version: u32,
    tool: &'a str,
    created_unix: u64,
    path_hmac_sha256: &'a str,
    content_sha256: &'a str,
    size: u64,
}

/// Shell-friendly wrapper requested in the design notes.
pub fn auth(action: ActionType, file_list: Vec<String>, options: AuthOptions) -> bool {
    auth_report(action, file_list, options).map(|r| r.ok()).unwrap_or(false)
}

/// Detailed API used by the CLI and suitable for library callers.
pub fn auth_report(
    action: ActionType,
    file_list: Vec<String>,
    options: AuthOptions,
) -> Result<AuthReport, AuthError> {
    ensure_db(&options.db_dir)?;
    if matches!(action, ActionType::Write | ActionType::Remove)
        && !options.force
        && options.authorization == AuthorizationMode::Platform
    {
        platform_authorize(&options.reason)?;
    }

    let mut report = AuthReport::default();
    let keys = DbKeys::load_or_create(&options.db_dir)?;

    for file in file_list {
        let path = PathBuf::from(&file);
        if !is_readable_file(&path) && action != ActionType::Remove {
            report.failed += 1;
            if options.verbose >= 0 {
                eprintln!("Error: unable to read {}", path.display());
            }
            continue;
        }

        match action {
            ActionType::Write => match write_record(&path, &options.db_dir, &keys) {
                Ok(()) => {
                    report.written += 1;
                    if options.verbose > 0 {
                        eprintln!("Info: authorized {}", path.display());
                    }
                }
                Err(e) => {
                    report.failed += 1;
                    if options.verbose >= 0 {
                        eprintln!("Error: {e}");
                    }
                }
            },
            ActionType::Check => {
                report.checked += 1;
                match check_record(&path, &options.db_dir, &keys) {
                    Ok(()) => {
                        report.passed += 1;
                        if options.verbose > 0 {
                            eprintln!("Info: {} passes", path.display());
                        }
                    }
                    Err(e) => {
                        report.failed += 1;
                        if options.verbose >= 0 {
                            eprintln!("Error: {e}");
                        }
                    }
                }
            }
            ActionType::Remove => match remove_record(&path, &options.db_dir, &keys) {
                Ok(true) => {
                    report.removed += 1;
                    if options.verbose > 0 {
                        eprintln!("Info: removed authorization for {}", path.display());
                    }
                }
                Ok(false) => {
                    report.skipped += 1;
                    if options.verbose >= 0 {
                        eprintln!("Warning: no authorization record for {}", path.display());
                    }
                }
                Err(e) => {
                    report.failed += 1;
                    if options.verbose >= 0 {
                        eprintln!("Error: {e}");
                    }
                }
            },
        }
    }
    Ok(report)
}

struct DbKeys {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    path_key: Vec<u8>,
}

impl DbKeys {
    fn load_or_create(db_dir: &Path) -> Result<Self, AuthError> {
        let signing_path = db_dir.join(KEYPAIR_FILE);
        let verifying_path = db_dir.join(PUBKEY_FILE);
        let path_key_path = db_dir.join(PATH_KEY_FILE);

        let signing_key = if signing_path.exists() {
            let bytes = fs::read(&signing_path)?;
            let arr: [u8; 32] = bytes.try_into().map_err(|_| AuthError::InvalidSigningKey)?;
            SigningKey::from_bytes(&arr)
        } else {
            let key = SigningKey::generate(&mut OsRng);
            write_secret_file(&signing_path, key.to_bytes().as_slice())?;
            key
        };

        let verifying_key = signing_key.verifying_key();
        if !verifying_path.exists() {
            fs::write(&verifying_path, verifying_key.to_bytes())?;
        }

        let path_key = if path_key_path.exists() {
            fs::read(&path_key_path)?
        } else {
            let mut key = vec![0u8; 32];
            OsRng.fill_bytes(&mut key);
            write_secret_file(&path_key_path, &key)?;
            key
        };

        Ok(Self {
            signing_key,
            verifying_key,
            path_key,
        })
    }
}

fn write_record(path: &Path, db_dir: &Path, keys: &DbKeys) -> Result<(), AuthError> {
    let canonical = canonicalize_existing(path)?;
    let path_hmac = path_hmac(&canonical, &keys.path_key)?;
    let digest = file_sha256(&canonical)?;
    let size = fs::metadata(&canonical)?.len();
    let created_unix = unix_now();
    let payload = SignedPayload {
        version: 1,
        tool: concat!("auth ", env!("CARGO_PKG_VERSION")),
        created_unix,
        path_hmac_sha256: &path_hmac,
        content_sha256: &digest,
        size,
    };
    let payload_bytes = serde_json::to_vec(&payload)?;
    let signature = keys.signing_key.sign(&payload_bytes);
    let record = AuthRecord {
        version: payload.version,
        tool: payload.tool.to_string(),
        created_unix,
        path_hmac_sha256: path_hmac.clone(),
        content_sha256: digest,
        size,
        signature: B64.encode(signature.to_bytes()),
    };
    let record_path = record_path(db_dir, &path_hmac);
    fs::write(record_path, serde_json::to_vec_pretty(&record)?)?;
    Ok(())
}

fn check_record(path: &Path, db_dir: &Path, keys: &DbKeys) -> Result<(), AuthError> {
    let canonical = canonicalize_existing(path)?;
    let path_hmac = path_hmac(&canonical, &keys.path_key)?;
    let record_path = record_path(db_dir, &path_hmac);
    if !record_path.exists() {
        return Err(AuthError::RecordMissing(path.to_path_buf()));
    }
    let record: AuthRecord = serde_json::from_slice(&fs::read(record_path)?)?;
    let digest = file_sha256(&canonical)?;
    let size = fs::metadata(&canonical)?.len();

    if record.path_hmac_sha256 != path_hmac {
        return Err(AuthError::ValidationFailed(path.to_path_buf(), "path HMAC mismatch".into()));
    }
    if record.content_sha256 != digest {
        return Err(AuthError::ValidationFailed(path.to_path_buf(), "content digest mismatch".into()));
    }
    if record.size != size {
        return Err(AuthError::ValidationFailed(path.to_path_buf(), "size mismatch".into()));
    }

    let payload = SignedPayload {
        version: record.version,
        tool: &record.tool,
        created_unix: record.created_unix,
        path_hmac_sha256: &record.path_hmac_sha256,
        content_sha256: &record.content_sha256,
        size: record.size,
    };
    let sig_bytes = B64.decode(record.signature.as_bytes()).map_err(|_| AuthError::InvalidSignature)?;
    let sig_arr: [u8; 64] = sig_bytes.try_into().map_err(|_| AuthError::InvalidSignature)?;
    let sig = Signature::from_bytes(&sig_arr);
    keys.verifying_key
        .verify(&serde_json::to_vec(&payload)?, &sig)
        .map_err(|_| AuthError::InvalidSignature)?;
    Ok(())
}

fn remove_record(path: &Path, db_dir: &Path, keys: &DbKeys) -> Result<bool, AuthError> {
    let canonical = if path.exists() { canonicalize_existing(path)? } else { path.to_path_buf() };
    let path_hmac = path_hmac(&canonical, &keys.path_key)?;
    let rp = record_path(db_dir, &path_hmac);
    if rp.exists() {
        fs::remove_file(rp)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn record_path(db_dir: &Path, path_hmac: &str) -> PathBuf {
    db_dir.join(format!("{path_hmac}.{RECORD_EXT}"))
}

fn path_hmac(path: &Path, key: &[u8]) -> Result<String, AuthError> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts arbitrary key sizes");
    mac.update(path.to_string_lossy().as_bytes());
    Ok(hex_lower(&mac.finalize().into_bytes()))
}

fn file_sha256(path: &Path) -> Result<String, AuthError> {
    let mut f = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, AuthError> {
    fs::canonicalize(path).map_err(|_| AuthError::FileNotReadable(path.to_path_buf()))
}

fn is_readable_file(path: &Path) -> bool {
    fs::File::open(path).is_ok()
}

fn ensure_db(db_dir: &Path) -> Result<(), AuthError> {
    fs::create_dir_all(db_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(db_dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn write_secret_file(path: &Path, bytes: &[u8]) -> Result<(), AuthError> {
    let mut file = fs::OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn default_db_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".auth")
}

pub fn platform_authorize(reason: &str) -> Result<(), AuthError> {
    platform::authorize(reason)
}

mod platform {
    use super::AuthError;

    #[cfg(target_os = "macos")]
    pub fn authorize(reason: &str) -> Result<(), AuthError> {
        // No GUI is built by auth itself. This invokes a tiny Swift helper that uses
        // LocalAuthentication and lets macOS present Touch ID / password fallback.
        // Install it as `auth-macos-touchid` somewhere on PATH.
        let status = std::process::Command::new("auth-macos-touchid")
            .arg(reason)
            .status()
            .map_err(|e| AuthError::UnsupportedAuthorization(format!(
                "could not invoke auth-macos-touchid helper: {e}. Build platform/macos/auth-macos-touchid.swift or use --no-platform-auth for development"
            )))?;
        if status.success() {
            Ok(())
        } else {
            Err(AuthError::AuthorizationDenied(format!("macOS LocalAuthentication helper failed: {status}")))
        }
    }

    #[cfg(target_os = "windows")]
    pub fn authorize(reason: &str) -> Result<(), AuthError> {
        use windows::Security::Credentials::UI::{UserConsentVerificationResult, UserConsentVerifier};
        use windows::core::HSTRING;

        let op = UserConsentVerifier::RequestVerificationAsync(&HSTRING::from(reason))
            .map_err(|e| AuthError::AuthorizationDenied(format!("Windows Hello request failed: {e}")))?;
        let result = op.get()
            .map_err(|e| AuthError::AuthorizationDenied(format!("Windows Hello failed: {e}")))?;
        match result {
            UserConsentVerificationResult::Verified => Ok(()),
            other => Err(AuthError::AuthorizationDenied(format!("Windows Hello result: {other:?}"))),
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn authorize(reason: &str) -> Result<(), AuthError> {
        // Minimal non-GUI Linux fallback: rely on sudo/PAM policy. This can be replaced with direct PAM.
        let status = std::process::Command::new("sudo")
            .arg("-v")
            .status()
            .map_err(|e| AuthError::AuthorizationDenied(format!("could not invoke sudo/PAM: {e}")))?;
        if status.success() {
            Ok(())
        } else {
            Err(AuthError::AuthorizationDenied(format!("PAM/sudo did not authorize: {reason}")))
        }
    }

    #[cfg(not(any(unix, windows)))]
    pub fn authorize(reason: &str) -> Result<(), AuthError> {
        Err(AuthError::UnsupportedAuthorization(format!("no platform backend for this OS: {reason}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn test_options(db: &Path) -> AuthOptions {
        AuthOptions {
            db_dir: db.to_path_buf(),
            verbose: -1,
            force: true,
            authorization: AuthorizationMode::None,
            reason: "test authorization".to_string(),
        }
    }

    #[test]
    fn write_then_check_passes() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("db");
        let file = tmp.path().join("secret-plan.txt");
        fs::write(&file, "approved contents\n").unwrap();

        let wr = auth_report(
            ActionType::Write,
            vec![file.to_string_lossy().into_owned()],
            test_options(&db),
        ).unwrap();
        assert!(wr.ok());
        assert_eq!(wr.written, 1);

        let ck = auth_report(
            ActionType::Check,
            vec![file.to_string_lossy().into_owned()],
            test_options(&db),
        ).unwrap();
        assert!(ck.ok());
        assert_eq!(ck.passed, 1);
    }

    #[test]
    fn changed_file_fails_check() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("db");
        let file = tmp.path().join("sensitive.txt");
        fs::write(&file, "before\n").unwrap();

        auth_report(ActionType::Write, vec![file.to_string_lossy().into_owned()], test_options(&db)).unwrap();
        fs::write(&file, "after\n").unwrap();
        let ck = auth_report(ActionType::Check, vec![file.to_string_lossy().into_owned()], test_options(&db)).unwrap();
        assert!(!ck.ok());
        assert_eq!(ck.failed, 1);
    }

    #[test]
    fn record_filename_does_not_expose_original_filename() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("db");
        let file = tmp.path().join("top-secret-customer-list.txt");
        fs::write(&file, "classified-ish\n").unwrap();

        auth_report(ActionType::Write, vec![file.to_string_lossy().into_owned()], test_options(&db)).unwrap();
        let entries: Vec<_> = fs::read_dir(&db).unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(entries.iter().any(|name| name.ends_with(".auth.json")));
        assert!(entries.iter().all(|name| !name.contains("top-secret-customer-list")));
    }

    #[test]
    fn remove_deletes_authorization_record_for_existing_file() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("db");
        let file = tmp.path().join("remove-me.txt");
        fs::write(&file, "remove me\n").unwrap();

        auth_report(ActionType::Write, vec![file.to_string_lossy().into_owned()], test_options(&db)).unwrap();
        let rm = auth_report(ActionType::Remove, vec![file.to_string_lossy().into_owned()], test_options(&db)).unwrap();
        assert!(rm.ok());
        assert_eq!(rm.removed, 1);

        let ck = auth_report(ActionType::Check, vec![file.to_string_lossy().into_owned()], test_options(&db)).unwrap();
        assert!(!ck.ok());
        assert_eq!(ck.failed, 1);
    }
}
