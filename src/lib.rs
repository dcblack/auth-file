//! `authlib` - file authorization and validation library.
//!
//! The crate intentionally separates authorization from validation:
//! - `Write` and `Remove` may require user authorization.
//! - `Check` verifies stored cryptographic records without prompting.
//!
//! Version 0.7.1 stores authorization records in `SQLite` and moves normal-use
//! secret keys into the platform credential store. Test databases named
//! `auth-test` keep file-backed keys for CI and development only.

#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const TEST_KEYPAIR_FILE: &str = "ed25519.signing-key";
const TEST_PATH_KEY_FILE: &str = "path-hmac.key";
const PUBKEY_FILE: &str = "ed25519.verifying-key";
const KEYRING_SERVICE: &str = "auth-file";
const SQLITE_FILE: &str = "auth.db";
const SCHEMA_VERSION: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Check,
    Write,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl ColorMode {
    fn enabled(self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => {
                std::io::stderr().is_terminal()
                    && std::env::var_os("NO_COLOR").is_none()
                    && std::env::var_os("NOCOLOR").is_none()
            }
        }
    }
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
    pub color: ColorMode,
}

impl Default for AuthOptions {
    fn default() -> Self {
        Self {
            db_dir: default_db_dir(),
            verbose: 0,
            force: false,
            authorization: AuthorizationMode::Platform,
            reason: "Authorize file trust database change".to_string(),
            color: ColorMode::Auto,
        }
    }
}

impl AuthOptions {
    #[must_use]
    pub fn colorize_error(&self, msg: &str) -> String {
        colorize(self.color, "31", msg)
    }

    #[must_use]
    pub fn colorize_warning(&self, msg: &str) -> String {
        colorize(self.color, "33", msg)
    }

    #[must_use]
    pub fn colorize_pass(&self, msg: &str) -> String {
        colorize(self.color, "32", msg)
    }
}

fn colorize(mode: ColorMode, code: &str, msg: &str) -> String {
    if mode.enabled() {
        format!("\x1b[{code}m{msg}\x1b[0m")
    } else {
        msg.to_string()
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
    #[must_use]
    pub const fn ok(&self) -> bool {
        self.failed == 0
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid signing key")]
    InvalidSigningKey,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("key storage error: {0}")]
    KeyStorage(String),
    #[error("key decoding error: {0}")]
    KeyDecode(String),
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

#[derive(Debug, Clone)]
struct AuthRecord {
    version: u32,
    tool: String,
    created_unix: u64,
    updated_unix: u64,
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
    updated_unix: u64,
    path_hmac_sha256: &'a str,
    content_sha256: &'a str,
    size: u64,
}

/// Shell-friendly wrapper requested in the design notes.
#[must_use]
pub fn auth(action: ActionType, file_list: Vec<String>, options: &AuthOptions) -> bool {
    auth_report(action, file_list, options).is_ok_and(|r| r.ok())
}

/// Detailed API used by the CLI and suitable for library callers.
///
/// # Errors
///
/// Returns an error when storage cannot be initialized, platform authorization
/// fails, key material cannot be loaded, or the underlying database operation
/// fails. Per-file validation failures are normally reported inside the returned
/// [`AuthReport`] so scripts can process multiple files in one invocation.
pub fn auth_report(
    action: ActionType,
    file_list: Vec<String>,
    options: &AuthOptions,
) -> Result<AuthReport, AuthError> {
    let database_was_missing = !database_path(&options.db_dir).exists();
    ensure_storage(&options.db_dir)?;
    if matches!(action, ActionType::Write | ActionType::Remove)
        && options.authorization == AuthorizationMode::Platform
    {
        platform_authorize(&options.reason)?;
    }

    let conn = open_database(&options.db_dir)?;
    let allow_key_create = database_was_missing || records_table_is_empty(&conn)?;
    let mut report = AuthReport::default();
    let keys = DbKeys::load_or_create(&options.db_dir, allow_key_create)?;

    for file in file_list {
        let path = PathBuf::from(&file);
        if !is_readable_file(&path) && action != ActionType::Remove {
            report.failed += 1;
            if options.verbose >= 0 {
                eprintln!(
                    "{}",
                    options.colorize_error(&format!("Error: unable to read {}", path.display()))
                );
            }
            continue;
        }

        match action {
            ActionType::Write => match write_record(&conn, &path, &keys) {
                Ok(()) => {
                    report.written += 1;
                    if options.verbose > 0 {
                        eprintln!(
                            "{}",
                            options.colorize_pass(&format!("Pass: authorized {}", path.display()))
                        );
                    }
                }
                Err(e) => {
                    report.failed += 1;
                    if options.verbose >= 0 {
                        eprintln!("{}", options.colorize_error(&format!("Error: {e}")));
                    }
                }
            },
            ActionType::Check => {
                report.checked += 1;
                match check_record(&conn, &path, &keys) {
                    Ok(()) => {
                        report.passed += 1;
                        if options.verbose > 0 {
                            eprintln!(
                                "{}",
                                options.colorize_pass(&format!("Pass: {} passes", path.display()))
                            );
                        }
                    }
                    Err(e) => {
                        report.failed += 1;
                        if options.verbose >= 0 {
                            eprintln!("{}", options.colorize_error(&format!("Error: {e}")));
                        }
                    }
                }
            }
            ActionType::Remove => match remove_record(&conn, &path, &keys) {
                Ok(true) => {
                    report.removed += 1;
                    if options.verbose > 0 {
                        eprintln!(
                            "{}",
                            options.colorize_pass(&format!(
                                "Pass: removed authorization for {}",
                                path.display()
                            ))
                        );
                    }
                }
                Ok(false) => {
                    report.skipped += 1;
                    if options.verbose >= 0 {
                        eprintln!(
                            "{}",
                            options.colorize_warning(&format!(
                                "Warning: no authorization record for {}",
                                path.display()
                            ))
                        );
                    }
                }
                Err(e) => {
                    report.failed += 1;
                    if options.verbose >= 0 {
                        eprintln!("{}", options.colorize_error(&format!("Error: {e}")));
                    }
                }
            },
        }
    }
    Ok(report)
}

struct DbKeys {
    signing: SigningKey,
    verifying: VerifyingKey,
    path_hmac_secret: Vec<u8>,
}

impl DbKeys {
    fn load_or_create(db_dir: &Path, allow_create: bool) -> Result<Self, AuthError> {
        if is_test_database_dir(db_dir) {
            return Self::load_or_create_test_files(db_dir, allow_create);
        }
        Self::load_or_create_keyring(db_dir, allow_create)
    }

    fn load_or_create_keyring(db_dir: &Path, allow_create: bool) -> Result<Self, AuthError> {
        let namespace = key_namespace(db_dir);
        let signing_name = format!("{namespace}:ed25519-signing");
        let path_name = format!("{namespace}:path-hmac");

        let signing_bytes = get_or_create_secret(&signing_name, allow_create, || {
            SigningKey::generate(&mut OsRng).to_bytes().to_vec()
        })?;
        let signing_array: [u8; 32] = signing_bytes
            .try_into()
            .map_err(|_| AuthError::InvalidSigningKey)?;
        let signing = SigningKey::from_bytes(&signing_array);
        let verifying = signing.verifying_key();

        write_public_file(&db_dir.join(PUBKEY_FILE), verifying.to_bytes().as_slice())?;

        let path_hmac_secret = get_or_create_secret(&path_name, allow_create, || {
            let mut key = vec![0_u8; 32];
            OsRng.fill_bytes(&mut key);
            key
        })?;

        Ok(Self {
            signing,
            verifying,
            path_hmac_secret,
        })
    }

    fn load_or_create_test_files(db_dir: &Path, allow_create: bool) -> Result<Self, AuthError> {
        let signing_path = db_dir.join(TEST_KEYPAIR_FILE);
        let verifying_path = db_dir.join(PUBKEY_FILE);
        let path_key_path = db_dir.join(TEST_PATH_KEY_FILE);

        let signing = if signing_path.exists() {
            let bytes = fs::read(&signing_path)?;
            let arr: [u8; 32] = bytes.try_into().map_err(|_| AuthError::InvalidSigningKey)?;
            SigningKey::from_bytes(&arr)
        } else if allow_create {
            let key = SigningKey::generate(&mut OsRng);
            write_secret_file(&signing_path, key.to_bytes().as_slice())?;
            key
        } else {
            return Err(AuthError::KeyStorage(format!(
                "database exists but test signing key is missing: {}",
                signing_path.display()
            )));
        };

        let verifying = signing.verifying_key();
        write_public_file(&verifying_path, verifying.to_bytes().as_slice())?;

        let path_hmac_secret = if path_key_path.exists() {
            fs::read(&path_key_path)?
        } else if allow_create {
            let mut key = vec![0_u8; 32];
            OsRng.fill_bytes(&mut key);
            write_secret_file(&path_key_path, &key)?;
            key
        } else {
            return Err(AuthError::KeyStorage(format!(
                "database exists but test path HMAC key is missing: {}",
                path_key_path.display()
            )));
        };

        Ok(Self {
            signing,
            verifying,
            path_hmac_secret,
        })
    }
}

fn write_record(conn: &Connection, path: &Path, keys: &DbKeys) -> Result<(), AuthError> {
    let canonical = canonicalize_existing(path)?;
    let path_hmac = path_hmac(&canonical, &keys.path_hmac_secret);
    let digest = file_sha256(&canonical)?;
    let size = fs::metadata(&canonical)?.len();
    let now = unix_now();
    let existing_created = existing_created_unix(conn, &path_hmac)?;
    let created_unix = existing_created.unwrap_or(now);
    let updated_unix = now;
    let tool = concat!("auth ", env!("CARGO_PKG_VERSION"));

    let payload = SignedPayload {
        version: 1,
        tool,
        created_unix,
        updated_unix,
        path_hmac_sha256: &path_hmac,
        content_sha256: &digest,
        size,
    };
    let payload_bytes = serde_json::to_vec(&payload)?;
    let signature = keys.signing.sign(&payload_bytes);

    conn.execute(
        r"
        INSERT INTO records (
            path_hmac_sha256,
            content_sha256,
            size,
            version,
            tool,
            created_unix,
            updated_unix,
            signature
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(path_hmac_sha256) DO UPDATE SET
            content_sha256 = excluded.content_sha256,
            size = excluded.size,
            version = excluded.version,
            tool = excluded.tool,
            updated_unix = excluded.updated_unix,
            signature = excluded.signature
        ",
        params![
            path_hmac,
            digest,
            i64::try_from(size).unwrap_or(i64::MAX),
            i64::from(payload.version),
            tool,
            i64::try_from(created_unix).unwrap_or(i64::MAX),
            i64::try_from(updated_unix).unwrap_or(i64::MAX),
            B64.encode(signature.to_bytes()),
        ],
    )?;
    Ok(())
}

fn check_record(conn: &Connection, path: &Path, keys: &DbKeys) -> Result<(), AuthError> {
    let canonical = canonicalize_existing(path)?;
    let path_hmac = path_hmac(&canonical, &keys.path_hmac_secret);
    let record = load_record(conn, &path_hmac)?
        .ok_or_else(|| AuthError::RecordMissing(path.to_path_buf()))?;
    let digest = file_sha256(&canonical)?;
    let size = fs::metadata(&canonical)?.len();

    if record.path_hmac_sha256 != path_hmac {
        return Err(AuthError::ValidationFailed(
            path.to_path_buf(),
            "path HMAC mismatch".into(),
        ));
    }
    if record.content_sha256 != digest {
        return Err(AuthError::ValidationFailed(
            path.to_path_buf(),
            "content digest mismatch".into(),
        ));
    }
    if record.size != size {
        return Err(AuthError::ValidationFailed(
            path.to_path_buf(),
            "size mismatch".into(),
        ));
    }

    verify_record(&record, keys)?;
    Ok(())
}

fn remove_record(conn: &Connection, path: &Path, keys: &DbKeys) -> Result<bool, AuthError> {
    let canonical = if path.exists() {
        canonicalize_existing(path)?
    } else {
        path.to_path_buf()
    };
    let path_hmac = path_hmac(&canonical, &keys.path_hmac_secret);
    let count = conn.execute(
        "DELETE FROM records WHERE path_hmac_sha256 = ?1",
        params![path_hmac],
    )?;
    Ok(count > 0)
}

fn verify_record(record: &AuthRecord, keys: &DbKeys) -> Result<(), AuthError> {
    let payload = SignedPayload {
        version: record.version,
        tool: &record.tool,
        created_unix: record.created_unix,
        updated_unix: record.updated_unix,
        path_hmac_sha256: &record.path_hmac_sha256,
        content_sha256: &record.content_sha256,
        size: record.size,
    };
    let sig_bytes = B64
        .decode(record.signature.as_bytes())
        .map_err(|_| AuthError::InvalidSignature)?;
    let sig_arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| AuthError::InvalidSignature)?;
    let sig = Signature::from_bytes(&sig_arr);
    keys.verifying
        .verify(&serde_json::to_vec(&payload)?, &sig)
        .map_err(|_| AuthError::InvalidSignature)?;
    Ok(())
}

fn existing_created_unix(conn: &Connection, path_hmac: &str) -> Result<Option<u64>, AuthError> {
    let value: Option<i64> = conn
        .query_row(
            "SELECT created_unix FROM records WHERE path_hmac_sha256 = ?1",
            params![path_hmac],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value.and_then(|v| u64::try_from(v).ok()))
}

fn load_record(conn: &Connection, path_hmac: &str) -> Result<Option<AuthRecord>, AuthError> {
    conn.query_row(
        r"
        SELECT
            version,
            tool,
            created_unix,
            updated_unix,
            path_hmac_sha256,
            content_sha256,
            size,
            signature
        FROM records
        WHERE path_hmac_sha256 = ?1
        ",
        params![path_hmac],
        |row| {
            let version_i: i64 = row.get(0)?;
            let created_i: i64 = row.get(2)?;
            let updated_i: i64 = row.get(3)?;
            let size_i: i64 = row.get(6)?;
            Ok(AuthRecord {
                version: u32::try_from(version_i).unwrap_or(0),
                tool: row.get(1)?,
                created_unix: u64::try_from(created_i).unwrap_or(0),
                updated_unix: u64::try_from(updated_i).unwrap_or(0),
                path_hmac_sha256: row.get(4)?,
                content_sha256: row.get(5)?,
                size: u64::try_from(size_i).unwrap_or(0),
                signature: row.get(7)?,
            })
        },
    )
    .optional()
    .map_err(AuthError::from)
}

fn path_hmac(path: &Path, key: &[u8]) -> String {
    let Ok(mut mac) = HmacSha256::new_from_slice(key) else {
        unreachable!("HMAC accepts arbitrary key sizes");
    };
    mac.update(path.to_string_lossy().as_bytes());
    hex_lower(mac.finalize().into_bytes())
}

fn file_sha256(path: &Path) -> Result<String, AuthError> {
    let mut f = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex_lower(hasher.finalize()))
}

fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}

fn get_or_create_secret(
    name: &str,
    allow_create: bool,
    generate: impl FnOnce() -> Vec<u8>,
) -> Result<Vec<u8>, AuthError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, name)
        .map_err(|e| AuthError::KeyStorage(e.to_string()))?;

    if let Ok(secret) = entry.get_password() {
        B64.decode(secret.as_bytes())
            .map_err(|e| AuthError::KeyDecode(e.to_string()))
    } else if allow_create {
        let secret = generate();
        entry
            .set_password(&B64.encode(&secret))
            .map_err(|e| AuthError::KeyStorage(e.to_string()))?;
        Ok(secret)
    } else {
        Err(AuthError::KeyStorage(format!(
            "database exists but credential-store secret is missing: {name}"
        )))
    }
}

fn key_namespace(db_dir: &Path) -> String {
    let stable_path = absolute_database_dir(db_dir);
    let digest = Sha256::digest(stable_path.to_string_lossy().as_bytes());
    format!("db-{}", hex_lower(digest))
}

fn absolute_database_dir(db_dir: &Path) -> PathBuf {
    if db_dir.is_absolute() {
        db_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(db_dir)
    }
}

fn is_test_database_dir(db_dir: &Path) -> bool {
    db_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "auth-test")
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, AuthError> {
    fs::canonicalize(path).map_err(|_| AuthError::FileNotReadable(path.to_path_buf()))
}

fn is_readable_file(path: &Path) -> bool {
    fs::File::open(path).is_ok()
}

fn ensure_storage(db_dir: &Path) -> Result<(), AuthError> {
    fs::create_dir_all(db_dir)?;

    #[cfg(unix)]
    set_private_dir_permissions(db_dir)?;

    #[cfg(not(unix))]
    set_private_dir_permissions(db_dir);

    let conn = open_database(db_dir)?;
    initialize_schema(&conn)?;

    #[cfg(unix)]
    set_private_file_permissions(&database_path(db_dir))?;

    #[cfg(not(unix))]
    set_private_file_permissions(&database_path(db_dir));

    Ok(())
}

fn open_database(db_dir: &Path) -> Result<Connection, AuthError> {
    let path = database_path(db_dir);
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    initialize_schema(&conn)?;
    Ok(conn)
}

fn initialize_schema(conn: &Connection) -> Result<(), AuthError> {
    conn.execute_batch(
        r"
        CREATE TABLE IF NOT EXISTS records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path_hmac_sha256 TEXT NOT NULL UNIQUE,
            content_sha256 TEXT NOT NULL,
            size INTEGER NOT NULL,
            version INTEGER NOT NULL,
            tool TEXT NOT NULL,
            created_unix INTEGER NOT NULL,
            updated_unix INTEGER NOT NULL,
            signature TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_records_path_hmac
            ON records(path_hmac_sha256);
        ",
    )?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

fn database_path(db_dir: &Path) -> PathBuf {
    db_dir.join(SQLITE_FILE)
}

fn records_table_is_empty(conn: &Connection) -> Result<bool, AuthError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))?;
    Ok(count == 0)
}

fn write_secret_file(path: &Path, bytes: &[u8]) -> Result<(), AuthError> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    file.write_all(bytes)?;
    set_private_file_permissions(path)?;
    Ok(())
}

fn write_public_file(path: &Path, bytes: &[u8]) -> Result<(), AuthError> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    file.write_all(bytes)?;
    set_private_file_permissions(path)?;
    Ok(())
}

//---------
#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> Result<(), AuthError> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions)?;

    Ok(())
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) {}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<(), AuthError> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(path, permissions)?;

    Ok(())
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) {}
//---------

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn default_db_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".auth")
}

/// Request authorization using the platform-specific backend.
///
/// # Errors
///
/// Returns an error when no backend is available, the helper/backend cannot be
/// invoked, or the platform reports a denial/cancel result.
pub fn platform_authorize(reason: &str) -> Result<(), AuthError> {
    platform::authorize(reason)
}

mod platform {
    use super::AuthError;

    #[cfg(target_os = "macos")]
    pub fn authorize(reason: &str) -> Result<(), AuthError> {
        use std::path::PathBuf;
        use std::process::Command;

        // No GUI is built by auth itself. This invokes a tiny Swift helper that uses
        // LocalAuthentication and lets macOS present Touch ID / password fallback.
        // Search order:
        //   1. AUTH_MACOS_TOUCHID_HELPER runtime override
        //   2. helper compiled by build.rs into OUT_DIR
        //   3. helper installed beside the auth executable
        //   4. helper on PATH
        let helper = std::env::var_os("AUTH_MACOS_TOUCHID_HELPER")
            .map(PathBuf::from)
            .or_else(|| {
                let built = env!("AUTH_BUILT_MACOS_HELPER");
                (!built.is_empty()).then(|| PathBuf::from(built))
            })
            .filter(|p| p.exists())
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|dir| dir.join("auth-macos-touchid")))
                    .filter(|p| p.exists())
            })
            .unwrap_or_else(|| PathBuf::from("auth-macos-touchid"));

        let status = Command::new(&helper)
            .arg(reason)
            .status()
            .map_err(|e| AuthError::UnsupportedAuthorization(format!(
                "could not invoke auth-macos-touchid helper at {}: {e}. Use --no-platform-auth for development/CI or set AUTH_MACOS_TOUCHID_HELPER.",
                helper.display()
            )))?;
        if status.success() {
            Ok(())
        } else {
            Err(AuthError::AuthorizationDenied(format!(
                "macOS LocalAuthentication helper failed: {status}"
            )))
        }
    }

    #[cfg(target_os = "windows")]
    pub fn authorize(reason: &str) -> Result<(), AuthError> {
        use windows::core::HSTRING;
        use windows::Security::Credentials::UI::{
            UserConsentVerificationResult, UserConsentVerifier,
        };

        let op =
            UserConsentVerifier::RequestVerificationAsync(&HSTRING::from(reason)).map_err(|e| {
                AuthError::AuthorizationDenied(format!("Windows Hello request failed: {e}"))
            })?;
        let result = op
            .get()
            .map_err(|e| AuthError::AuthorizationDenied(format!("Windows Hello failed: {e}")))?;
        match result {
            UserConsentVerificationResult::Verified => Ok(()),
            other => Err(AuthError::AuthorizationDenied(format!(
                "Windows Hello result: {other:?}"
            ))),
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn authorize(reason: &str) -> Result<(), AuthError> {
        // Minimal non-GUI Linux fallback: rely on sudo/PAM policy. This can be replaced with direct PAM.
        let status = std::process::Command::new("sudo")
            .arg("-v")
            .status()
            .map_err(|e| {
                AuthError::AuthorizationDenied(format!("could not invoke sudo/PAM: {e}"))
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(AuthError::AuthorizationDenied(format!(
                "PAM/sudo did not authorize: {reason}"
            )))
        }
    }

    #[cfg(not(any(unix, windows)))]
    pub fn authorize(reason: &str) -> Result<(), AuthError> {
        Err(AuthError::UnsupportedAuthorization(format!(
            "no platform backend for this OS: {reason}"
        )))
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
            color: ColorMode::Never,
        }
    }

    #[test]
    fn write_then_check_passes() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("auth-test");
        let file = tmp.path().join("secret-plan.txt");
        fs::write(&file, "approved contents\n").unwrap();

        let wr = auth_report(
            ActionType::Write,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();
        assert!(wr.ok());
        assert_eq!(wr.written, 1);
        assert!(db.join(SQLITE_FILE).exists());

        let ck = auth_report(
            ActionType::Check,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();
        assert!(ck.ok());
        assert_eq!(ck.passed, 1);
    }

    #[test]
    fn changed_file_fails_check() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("auth-test");
        let file = tmp.path().join("sensitive.txt");
        fs::write(&file, "before\n").unwrap();

        auth_report(
            ActionType::Write,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();
        fs::write(&file, "after\n").unwrap();
        let ck = auth_report(
            ActionType::Check,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();
        assert!(!ck.ok());
        assert_eq!(ck.failed, 1);
    }

    #[test]
    fn sqlite_database_does_not_store_plain_filename() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("auth-test");
        let file = tmp.path().join("top-secret-customer-list.txt");
        fs::write(&file, "classified-ish\n").unwrap();

        auth_report(
            ActionType::Write,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();
        let db_bytes = fs::read(db.join(SQLITE_FILE)).unwrap();
        let db_text = String::from_utf8_lossy(&db_bytes);
        assert!(!db_text.contains("top-secret-customer-list"));
        assert!(!db_text.contains(file.to_string_lossy().as_ref()));
    }

    #[test]
    fn remove_deletes_authorization_record_for_existing_file() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("auth-test");
        let file = tmp.path().join("remove-me.txt");
        fs::write(&file, "remove me\n").unwrap();

        auth_report(
            ActionType::Write,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();
        let rm = auth_report(
            ActionType::Remove,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();
        assert!(rm.ok());
        assert_eq!(rm.removed, 1);

        let ck = auth_report(
            ActionType::Check,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();
        assert!(!ck.ok());
        assert_eq!(ck.failed, 1);
    }

    #[test]
    fn first_run_bootstrap_creates_database_and_test_keys() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("auth-test");
        let file = tmp.path().join("bootstrap.txt");
        fs::write(&file, "bootstrap contents\n").unwrap();

        assert!(!db.exists());

        let wr = auth_report(
            ActionType::Write,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();

        assert!(wr.ok());
        assert!(db.join(SQLITE_FILE).exists());
        assert!(db.join(TEST_KEYPAIR_FILE).exists());
        assert!(db.join(TEST_PATH_KEY_FILE).exists());
        assert!(db.join(PUBKEY_FILE).exists());
    }

    #[test]
    fn existing_database_reuses_test_keys() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("auth-test");
        let file = tmp.path().join("reuse.txt");
        fs::write(&file, "stable contents\n").unwrap();

        auth_report(
            ActionType::Write,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();

        let signing_before = fs::read(db.join(TEST_KEYPAIR_FILE)).unwrap();
        let path_key_before = fs::read(db.join(TEST_PATH_KEY_FILE)).unwrap();

        let ck = auth_report(
            ActionType::Check,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();

        assert!(ck.ok());
        assert_eq!(
            signing_before,
            fs::read(db.join(TEST_KEYPAIR_FILE)).unwrap()
        );
        assert_eq!(
            path_key_before,
            fs::read(db.join(TEST_PATH_KEY_FILE)).unwrap()
        );
    }

    #[test]
    fn corrupted_database_returns_error() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("auth-test");
        fs::create_dir_all(&db).unwrap();
        fs::write(db.join(SQLITE_FILE), "not a sqlite database\n").unwrap();
        let file = tmp.path().join("corrupt.txt");
        fs::write(&file, "contents\n").unwrap();

        let err = auth_report(
            ActionType::Check,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap_err();

        assert!(matches!(err, AuthError::Sqlite(_)));
    }

    #[test]
    fn existing_database_missing_test_key_fails_instead_of_recreating() {
        let tmp = tempdir().unwrap();
        let db = tmp.path().join("auth-test");
        let file = tmp.path().join("missing-key.txt");
        fs::write(&file, "contents\n").unwrap();

        auth_report(
            ActionType::Write,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap();

        fs::remove_file(db.join(TEST_PATH_KEY_FILE)).unwrap();

        let err = auth_report(
            ActionType::Check,
            vec![file.to_string_lossy().into_owned()],
            &test_options(&db),
        )
        .unwrap_err();

        assert!(matches!(err, AuthError::KeyStorage(_)));
    }
}
