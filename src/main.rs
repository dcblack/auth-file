#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use authlib::{
    auth_report, auth_stats, auth_storage_paths, change_fallback_password, set_runtime_env_default,
    ActionType, AuthOptions, AuthorizationMode, ColorMode, SecretProvider, VERSION,
};
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

const HELP: &str = r#"Name
----

  auth - authorize and validate files before scripts or automation use them

Synopsis
--------

  auth --help
  auth --version
  auth --write  [OPTIONS] FILENAME(S)
  auth --check  [OPTIONS] FILENAME(S)
  auth --remove [OPTIONS] FILENAME(S)
  auth --change-password [OPTIONS]
  auth --show-dir [OPTIONS]
  auth --stats [OPTIONS]
  auth --show-config [OPTIONS]

Options
-------

  --cache-time=SECONDS       Cache successful authorization for 0-120 seconds [default: 0]
  --change-password          Change Auth password using current password or burner[^2]
  --check, -ck               Verify specified files; read-only and does not require authorization
  --color=WHEN               Color output: auto, always, never [default: auto]
  --config=FILE              Read TOML configuration from FILE [default: ~/.auth.toml]
  --config=                  Disable configuration loading for this invocation
  --default-root             Use default full-path file identity
  --dir=DIR, -d DIR          Specify database directory [default: ~/.auth]
  --force, -f                Reserved for future non-security confirmation prompts
  --help, -h                 Display this text using a pager when interactive
  --quiet, -q                Report failures only
  --remove, -rm              Remove authorization records[^1]
  --request-password         Prefer Auth password/burner instead of platform authorization
  --root-dir=PATH            Store/check paths relative to this canonical root
  --secret-provider=PROVIDER Secret provider: prompt, env, os-keyring, 1password, bitwarden
  --secret-ref=REF          Provider-specific secret reference, such as op://VAULT/ITEM/FIELD
  --show-config              Display effective configuration after authorization[^2]
  --show-dir                 Display auth storage paths[^2]
  --silent, -s               Silent even with failure, useful in scripts
  --stats                    Display database statistics[^2]
  --verbose, -v              Increase verbosity
  --version                  Display version and build metadata
  --write, -wr               Authorize files[^1]

[^1]: Requires platform authorization or a valid cached authorization session.

[^2]: Requires platform authorization, cached authorization session, or Auth password.

Configuration
-------------

  Configuration is loaded in this order:

    1. TOML config file
    2. AUTH_OPTIONS environment variable
    3. command-line arguments

  The default config path is ~/.auth.toml. Use --config=FILE to select another file.
  Use --config= to disable config loading. This is useful when authorizing or
  repairing the config file itself.

  Example TOML:

    version = 1
    default_root = true
    cache_time = 60
    color = "always"
    secret_provider = "1password"
    secret_ref = "op://VAULT/ITEM/FIELD"

    # Escape hatch for options without first-class TOML keys:
    options = ["--verbose"]

  Supported structured TOML keys:

    version, options, AUTH_OPTIONS, cache_time, color, default_root, dir, db_dir,
    force, quiet, request_password, root_dir, secret_provider, secret_ref, silent, verbose

  Supported config/environment variables:

    AUTH_OPTIONS
    AUTH_TEST_FALLBACK_PASSWORD
    AUTH_TEST_FALLBACK_PASSWORD_CONFIRM
    AUTH_TEST_CURRENT_PASSWORD_OR_BURNER
    AUTH_MACOS_TOUCHID_HELPER
    AUTH_CONFIG_DISABLE
    NO_COLOR, NOCOLOR
    PAGER

  AUTH_TEST_* variables are honored only when the database directory basename
  is exactly auth-test.

Root Directives
---------------

  --default-root and --root-dir=PATH are root directives. At most one root
  directive may appear across the config file, AUTH_OPTIONS, and command line.
  A second root directive fails with:

    Error: Attempt to specify root directory more than once.

Secret Providers
----------------

  Supported provider names and aliases:

    prompt
    env, environment
    os-keyring, keyring, keys, oskeyring
    1password, 1p, 1pw
    bitwarden, bw

  1Password support reads the configured secret reference using `op read REF`.
  Example: --secret-ref="op://Private/auth-file/password"

Compatibility
-------------

  Options may appear between filenames, allowing a mix of actions in one call.
  Long options that take values prefer --name=value syntax. Short options such
  as -d still take their value as the next argument.

Examples
--------

    auth --write a.txt b.txt
    auth --check a.txt b.txt
    auth --request-password --cache-time=60 --root-dir=. --write a.txt b.txt
    auth --secret-provider=1p --secret-ref="op://Private/auth-file/{name}" --write a.txt
    auth --default-root --check setup.profile

Exit Status
-----------

  0  all requested operations succeeded
  1  at least one requested operation failed
  2  command-line usage error
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandMode {
    FileActions,
    ChangePassword,
    ShowDir,
    Stats,
    ShowConfig,
}

const ROOT_SPECIFIED_MORE_THAN_ONCE: &str = "Attempt to specify root directory more than once.";
const SECRET_PROVIDER_MORE_THAN_ONCE: &str = "Attempt to specify secret provider more than once.";
const SECRET_REF_MORE_THAN_ONCE: &str = "Attempt to specify secret reference more than once.";
const DEFAULT_CONFIG_FILE: &str = ".auth.toml";
const DISABLE_CONFIG_ENV: &str = "AUTH_CONFIG_DISABLE";

fn main() -> ExitCode {
    match run() {
        Ok(ok) => {
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(msg) => {
            eprintln!("Error: {msg}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool, String> {
    let prepared = collect_args()?;
    if handle_top_level_command(&prepared.args) {
        return Ok(true);
    }
    execute_args(&prepared.args, prepared.config_info)
}

fn print_version() {
    println!("auth {VERSION}");
    println!("build: {}", env!("AUTH_BUILD_KIND"));
    println!("branch: {}", built_info::BRANCH);
    println!("commit: {}", built_info::SHORT_COMMIT);
    println!("target: {}", built_info::BUILD_TARGET);
    println!("rustc: {}", built_info::RUST_VERSION);
    println!("built: {}", built_info::BUILD_TIME);
}

fn handle_top_level_command(args: &[String]) -> bool {
    if args.is_empty() {
        print_help();
        return true;
    }
    if args[0] == "--help" || args[0] == "-h" {
        print_help();
        return true;
    }
    if args[0] == "--version" {
        print_version();
        return true;
    }
    false
}

struct CliState {
    action: ActionType,
    options: AuthOptions,
    overall_ok: bool,
    current_files: Vec<String>,
    mode: CommandMode,
    seen: SeenOptions,
    config_info: ConfigInfo,
}

#[derive(Default)]
struct SeenOptions {
    root_directive_seen: bool,
    secret_provider_seen: bool,
    secret_ref_seen: bool,
}

impl Default for CliState {
    fn default() -> Self {
        Self {
            action: ActionType::Check,
            options: AuthOptions::default(),
            overall_ok: true,
            current_files: Vec::new(),
            mode: CommandMode::FileActions,
            seen: SeenOptions::default(),
            config_info: ConfigInfo::default(),
        }
    }
}

fn execute_args(args: &[String], config_info: ConfigInfo) -> Result<bool, String> {
    let mut state = CliState {
        config_info,
        ..CliState::default()
    };
    let mut i = 0;
    while i < args.len() {
        parse_one_arg(args, &mut i, &mut state)?;
        i += 1;
    }
    finalize_cli_state(&mut state)
}

fn parse_one_arg(args: &[String], i: &mut usize, state: &mut CliState) -> Result<(), String> {
    let arg = args[*i].as_str();
    match arg {
        "-ck" | "--check" => switch_action(state, ActionType::Check),
        "-wr" | "--write" => switch_action(state, ActionType::Write),
        "-rm" | "--remove" => switch_action(state, ActionType::Remove),
        "--change-password" => set_mode(state, CommandMode::ChangePassword)?,
        unknown if unknown.starts_with("--color=") => {
            let Some((_, mode)) = unknown.split_once('=') else {
                return Err("--color requires --color=WHEN syntax".to_string());
            };
            state.options.color = parse_color_mode(mode)?;
        }
        "--color" => {
            *i += 1;
            let mode = args
                .get(*i)
                .ok_or_else(|| "missing value after --color".to_string())?;
            state.options.color = parse_color_mode(mode)?;
        }
        unknown if unknown.starts_with("--config=") => {}
        "--config" => return Err("use --config=FILE".to_string()),
        "--default-root" => {
            note_root_directive(state)?;
            state.options.root_dir = None;
        }
        "-d" => {
            *i += 1;
            let dir = args
                .get(*i)
                .ok_or_else(|| "missing directory after -d".to_string())?;
            state.options.db_dir = PathBuf::from(dir);
        }
        unknown if unknown.starts_with("--dir=") => {
            let Some((_, dir)) = unknown.split_once('=') else {
                return Err("--dir requires --dir=DIR syntax".to_string());
            };
            state.options.db_dir = PathBuf::from(dir);
        }
        "--dir" => {
            *i += 1;
            let dir = args
                .get(*i)
                .ok_or_else(|| "missing directory after --dir".to_string())?;
            state.options.db_dir = PathBuf::from(dir);
        }
        "-f" | "--force" => state.options.force = true,
        "--help" | "-h" => return Err("--help must be the first option".to_string()),
        "-q" | "--quiet" => state.options.verbose = 0,
        "--request-password" => state.options.authorization = AuthorizationMode::Password,
        unknown if unknown.starts_with("--secret-provider=") => {
            note_secret_provider(state)?;
            let Some((_, provider)) = unknown.split_once('=') else {
                return Err("--secret-provider requires --secret-provider=NAME syntax".to_string());
            };
            state.options.secret_provider = parse_secret_provider(provider)?;
        }
        unknown if unknown.starts_with("--secret-ref=") => {
            note_secret_ref(state)?;
            let Some((_, reference)) = unknown.split_once('=') else {
                return Err("--secret-ref requires --secret-ref=REF syntax".to_string());
            };
            state.options.secret_ref = if reference.is_empty() {
                None
            } else {
                Some(reference.to_string())
            };
        }
        "--secret-ref" => return Err("use --secret-ref=REF".to_string()),
        unknown if unknown.starts_with("--root-dir=") => {
            note_root_directive(state)?;
            let Some((_, root)) = unknown.split_once('=') else {
                return Err("--root-dir requires --root-dir=PATH syntax".to_string());
            };
            state.options.root_dir = if root.is_empty() {
                None
            } else {
                Some(PathBuf::from(root))
            };
        }
        "--root-dir" => return Err("use --root-dir=PATH".to_string()),
        "--show-config" => set_mode(state, CommandMode::ShowConfig)?,
        "--show-dir" => set_mode(state, CommandMode::ShowDir)?,
        "-s" | "--silent" => state.options.verbose = -1,
        "--stats" => set_mode(state, CommandMode::Stats)?,
        "-v" | "--verbose" => state.options.verbose = 1,
        "--version" => return Err("--version must be the first option".to_string()),
        unknown if unknown.starts_with("--cache-time=") => {
            let Some((_, seconds)) = unknown.split_once('=') else {
                return Err("--cache-time requires --cache-time=SECONDS syntax".to_string());
            };
            state.options.cache_seconds = parse_cache_time(seconds)?;
        }
        "--cache-time" => return Err("use --cache-time=SECONDS".to_string()),
        unknown if unknown.starts_with('-') => return Err(format!("unknown option {unknown}")),
        filename => state.current_files.push(filename.to_string()),
    }
    Ok(())
}

fn note_secret_provider(state: &mut CliState) -> Result<(), String> {
    if state.seen.secret_provider_seen {
        return Err(SECRET_PROVIDER_MORE_THAN_ONCE.to_string());
    }
    state.seen.secret_provider_seen = true;
    Ok(())
}

fn note_secret_ref(state: &mut CliState) -> Result<(), String> {
    if state.seen.secret_ref_seen {
        return Err(SECRET_REF_MORE_THAN_ONCE.to_string());
    }
    state.seen.secret_ref_seen = true;
    Ok(())
}

fn parse_secret_provider(value: &str) -> Result<SecretProvider, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "prompt" => Ok(SecretProvider::Prompt),
        "1password" | "1p" | "1pw" => Ok(SecretProvider::OnePassword),
        "bitwarden" | "bw" => Ok(SecretProvider::Bitwarden),
        "env" | "environment" => Ok(SecretProvider::Env),
        "keyring" | "keys" | "oskeyring" | "os-keyring" => Ok(SecretProvider::OsKeyring),
        other => Err(format!("unknown secret provider: {other}")),
    }
}

fn note_root_directive(state: &mut CliState) -> Result<(), String> {
    if state.seen.root_directive_seen {
        return Err(ROOT_SPECIFIED_MORE_THAN_ONCE.to_string());
    }
    state.seen.root_directive_seen = true;
    Ok(())
}

fn set_mode(state: &mut CliState, mode: CommandMode) -> Result<(), String> {
    if state.mode != CommandMode::FileActions && state.mode != mode {
        return Err("only one non-file command may be requested".to_string());
    }
    state.mode = mode;
    Ok(())
}

fn switch_action(state: &mut CliState, action: ActionType) {
    flush_current_files(state);
    state.action = action;
}

fn finalize_cli_state(state: &mut CliState) -> Result<bool, String> {
    match state.mode {
        CommandMode::FileActions => {
            flush_current_files(state);
            Ok(state.overall_ok)
        }
        CommandMode::ChangePassword => change_password(state),
        CommandMode::ShowDir => show_dir(state),
        CommandMode::Stats => show_stats(state),
        CommandMode::ShowConfig => show_config(state),
    }
}

fn change_password(state: &CliState) -> Result<bool, String> {
    reject_file_operands(state, "--change-password")?;
    let update = change_fallback_password(&state.options).map_err(|e| e.to_string())?;
    eprintln!(
        "{}",
        state.options.colorize_warning(
            "CRITICAL: Burner passwords were written to an age-encrypted file. Decrypt it and store them somewhere safe."
        )
    );
    eprintln!("Burner file: {}", update.burner_file.display());
    eprintln!("If you forget the Auth password, this file cannot help you recover.");
    Ok(true)
}

fn show_dir(state: &CliState) -> Result<bool, String> {
    reject_file_operands(state, "--show-dir")?;
    let paths = auth_storage_paths(&state.options).map_err(|e| e.to_string())?;
    println!("Auth directory: {}", paths.auth_dir.display());
    println!("Database: {}", paths.database.display());
    Ok(true)
}

fn show_config(state: &CliState) -> Result<bool, String> {
    reject_file_operands(state, "--show-config")?;

    // Showing the effective configuration reveals security-sensitive details
    // such as the auth database path and secret-provider reference, so it uses
    // the same protected path as --show-dir and --stats.
    let paths = auth_storage_paths(&state.options).map_err(|e| e.to_string())?;

    println!("Configuration:");
    match (&state.config_info.source, &state.config_info.path) {
        (ConfigSource::Disabled, _) => println!("  file: disabled (--config=)"),
        (_, Some(path)) => {
            println!("  file: {}", path.display());
            println!("  source: {}", state.config_info.source.label());
            println!(
                "  loaded: {}",
                if state.config_info.loaded {
                    "yes"
                } else {
                    "no"
                }
            );
        }
        (_, None) => println!("  file: none"),
    }

    println!();
    println!("Effective settings:");
    println!("  auth_dir: {}", paths.auth_dir.display());
    println!("  database: {}", paths.database.display());
    println!("  cache_time: {}", state.options.cache_seconds);
    println!("  color: {}", color_mode_name(state.options.color));
    println!(
        "  authorization: {}",
        match state.options.authorization {
            AuthorizationMode::Platform => "platform",
            AuthorizationMode::Password => "auth-password",
            AuthorizationMode::None => "none",
        }
    );
    println!(
        "  root: {}",
        state
            .options
            .root_dir
            .as_ref()
            .map_or_else(|| "default".to_string(), |path| path.display().to_string())
    );
    println!(
        "  secret_provider: {}",
        secret_provider_name(state.options.secret_provider)
    );
    println!(
        "  secret_ref: {}",
        state.options.secret_ref.as_deref().unwrap_or("(none)")
    );
    println!("  verbose: {}", state.options.verbose);
    println!("  force: {}", state.options.force);

    Ok(true)
}

fn color_mode_name(mode: ColorMode) -> &'static str {
    match mode {
        ColorMode::Auto => "auto",
        ColorMode::Always => "always",
        ColorMode::Never => "never",
    }
}

fn secret_provider_name(provider: SecretProvider) -> &'static str {
    match provider {
        SecretProvider::Prompt => "prompt",
        SecretProvider::Env => "env",
        SecretProvider::OsKeyring => "os-keyring",
        SecretProvider::OnePassword => "1password",
        SecretProvider::Bitwarden => "bitwarden",
    }
}

fn show_stats(state: &CliState) -> Result<bool, String> {
    reject_file_operands(state, "--stats")?;
    let auth_statistics = auth_stats(&state.options).map_err(|e| e.to_string())?;
    println!("Auth directory: {}", auth_statistics.auth_dir.display());
    println!("Database: {}", auth_statistics.database.display());
    println!("Authorized file entries: {}", auth_statistics.entries);
    println!(
        "Most recent write: {}",
        auth_statistics.last_write_utc.as_deref().unwrap_or("never")
    );
    println!(
        "Most recent check: {}",
        auth_statistics.last_check_utc.as_deref().unwrap_or("never")
    );
    Ok(true)
}

fn reject_file_operands(state: &CliState, option: &str) -> Result<(), String> {
    if state.current_files.is_empty() {
        Ok(())
    } else {
        Err(format!("{option} cannot be mixed with file operands"))
    }
}

fn flush_current_files(state: &mut CliState) {
    if state.current_files.is_empty() {
        return;
    }
    let batch = std::mem::take(&mut state.current_files);
    match auth_report(state.action, batch, &state.options) {
        Ok(report) => state.overall_ok &= report.ok(),
        Err(e) => {
            if state.options.verbose >= 0 {
                eprintln!("{}", state.options.colorize_error(&format!("Error: {e}")));
            }
            state.overall_ok = false;
        }
    }
}

#[derive(Debug, Clone)]
struct PreparedInput {
    args: Vec<String>,
    config_info: ConfigInfo,
}

#[derive(Debug, Clone)]
struct ConfigInfo {
    source: ConfigSource,
    path: Option<PathBuf>,
    loaded: bool,
}

impl Default for ConfigInfo {
    fn default() -> Self {
        Self {
            source: ConfigSource::Default,
            path: default_config_path(),
            loaded: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigSource {
    Default,
    CommandLine,
    Environment,
    Disabled,
}

impl ConfigSource {
    const fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::CommandLine => "command line",
            Self::Environment => "AUTH_OPTIONS",
            Self::Disabled => "disabled",
        }
    }
}

fn collect_args() -> Result<PreparedInput, String> {
    // Build one argument stream in the security-relevant precedence order:
    // config file first, then AUTH_OPTIONS, then the real command line. Help and
    // version are intentionally checked before loading config so a bad ~/.auth.toml
    // cannot prevent a user from asking for usage/version information.
    let cli_args: Vec<String> = env::args().skip(1).collect();

    if cli_args
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        return Ok(PreparedInput {
            args: cli_args,
            config_info: ConfigInfo::default(),
        });
    }

    if cli_args.first().is_some_and(|arg| arg == "--version") {
        return Ok(PreparedInput {
            args: cli_args,
            config_info: ConfigInfo::default(),
        });
    }

    let env_args = if let Ok(auth_options) = env::var("AUTH_OPTIONS") {
        split_auth_options(&auth_options)?
    } else {
        Vec::new()
    };

    let (config, config_info) = load_config_args(&cli_args, &env_args)?;
    apply_config_environment(&config.variables);

    let mut args = config.args;
    args.extend(env_args);
    args.extend(cli_args);
    Ok(PreparedInput { args, config_info })
}

struct EffectiveConfig {
    // Options synthesized from TOML are converted into the same canonical tokens
    // as CLI/AUTH_OPTIONS. That keeps validation rules, such as duplicate root
    // directives, centralized in the normal parser.
    args: Vec<String>,
    // A small compatibility layer for existing test/helper environment variables.
    // These are applied only when the process environment did not already set one.
    variables: Vec<(String, String)>,
}

impl EffectiveConfig {
    const fn empty() -> Self {
        Self {
            args: Vec::new(),
            variables: Vec::new(),
        }
    }
}

fn load_config_args(
    cli_args: &[String],
    env_args: &[String],
) -> Result<(EffectiveConfig, ConfigInfo), String> {
    if env::var_os(DISABLE_CONFIG_ENV).is_some() {
        return Ok((
            EffectiveConfig::empty(),
            ConfigInfo {
                source: ConfigSource::Disabled,
                path: None,
                loaded: false,
            },
        ));
    }

    let Some((path, source)) = config_path(cli_args, env_args) else {
        return Ok((
            EffectiveConfig::empty(),
            ConfigInfo {
                source: ConfigSource::Disabled,
                path: None,
                loaded: false,
            },
        ));
    };

    if !path.exists() {
        if explicit_config_requested(cli_args) || explicit_config_requested(env_args) {
            return Err(format!("configuration file not found: {}", path.display()));
        }
        return Ok((
            EffectiveConfig::empty(),
            ConfigInfo {
                source,
                path: Some(path),
                loaded: false,
            },
        ));
    }

    let config = read_config_file(&path)?;
    Ok((
        config,
        ConfigInfo {
            source,
            path: Some(path),
            loaded: true,
        },
    ))
}

fn config_path(cli_args: &[String], env_args: &[String]) -> Option<(PathBuf, ConfigSource)> {
    if config_disabled(cli_args) {
        return None;
    }
    if let Some(path) = explicit_config_path(cli_args) {
        return Some((path, ConfigSource::CommandLine));
    }
    if config_disabled(env_args) {
        return None;
    }
    if let Some(path) = explicit_config_path(env_args) {
        return Some((path, ConfigSource::Environment));
    }
    default_config_path().map(|path| (path, ConfigSource::Default))
}

fn config_disabled(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--config=")
}

fn explicit_config_path(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .filter_map(|arg| arg.strip_prefix("--config="))
        .find(|path| !path.is_empty())
        .map(PathBuf::from)
}

fn explicit_config_requested(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg.starts_with("--config=") && arg != "--config=")
}

fn default_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(DEFAULT_CONFIG_FILE))
}

fn read_config_file(path: &Path) -> Result<EffectiveConfig, String> {
    // The config file is a TOML document, not a shell fragment. Structured keys
    // are preferred, but `options = [...]` remains available for new CLI options
    // that have not yet been given first-class TOML keys.
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read configuration file {}: {e}", path.display()))?;
    let table: toml::Table = toml::from_str(&text).map_err(|e| {
        format!(
            "failed to parse TOML configuration file {}: {e}",
            path.display()
        )
    })?;

    let mut args = Vec::new();
    let mut variables = Vec::new();

    for (name, value) in &table {
        match name.as_str() {
            "version" => validate_config_version(value)?,
            "options" | "AUTH_OPTIONS" => extend_config_options(name, value, &mut args)?,
            "AUTH_TEST_FALLBACK_PASSWORD"
            | "AUTH_TEST_FALLBACK_PASSWORD_CONFIRM"
            | "AUTH_TEST_CURRENT_PASSWORD_OR_BURNER"
            | "AUTH_MACOS_TOUCHID_HELPER" => {
                let value = config_string(name, value)?;
                variables.push((name.clone(), value));
            }
            "cache_time" => {
                args.push(format!("--cache-time={}", config_u64(name, value)?));
            }
            "color" => {
                args.push(format!("--color={}", config_string(name, value)?));
            }
            "default_root" => {
                if config_bool(name, value)? {
                    args.push("--default-root".to_string());
                }
            }
            "dir" | "db_dir" => {
                args.push(format!("--dir={}", config_string(name, value)?));
            }
            "force" => {
                if config_bool(name, value)? {
                    args.push("--force".to_string());
                }
            }
            "quiet" => {
                if config_bool(name, value)? {
                    args.push("--quiet".to_string());
                }
            }
            "request_password" => {
                if config_bool(name, value)? {
                    args.push("--request-password".to_string());
                }
            }
            "root_dir" => {
                let root = config_string(name, value)?;
                args.push(format!("--root-dir={root}"));
            }
            "secret_provider" => {
                args.push(format!("--secret-provider={}", config_string(name, value)?));
            }
            "secret_ref" => {
                args.push(format!("--secret-ref={}", config_string(name, value)?));
            }
            "silent" => {
                if config_bool(name, value)? {
                    args.push("--silent".to_string());
                }
            }
            "verbose" => match value {
                toml::Value::Boolean(true) => args.push("--verbose".to_string()),
                toml::Value::Integer(n) if *n > 0 => args.push("--verbose".to_string()),
                toml::Value::Boolean(false) | toml::Value::Integer(0) => {}
                toml::Value::Integer(n) if *n < 0 => args.push("--silent".to_string()),
                _ => {
                    return Err(format!(
                        "configuration key {name} must be a boolean or integer"
                    ))
                }
            },
            other => return Err(format!("unsupported configuration key {other}")),
        }
    }

    Ok(EffectiveConfig { args, variables })
}

fn validate_config_version(value: &toml::Value) -> Result<(), String> {
    let Some(version) = value.as_integer() else {
        return Err("configuration version must be an integer".to_string());
    };
    if version == 1 {
        Ok(())
    } else {
        Err(format!("unsupported configuration version {version}"))
    }
}

fn config_string(name: &str, value: &toml::Value) -> Result<String, String> {
    value
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| format!("configuration key {name} must be a string"))
}

fn config_bool(name: &str, value: &toml::Value) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| format!("configuration key {name} must be a boolean"))
}

fn config_u64(name: &str, value: &toml::Value) -> Result<u64, String> {
    let Some(value) = value.as_integer() else {
        return Err(format!("configuration key {name} must be an integer"));
    };
    u64::try_from(value).map_err(|_| format!("configuration key {name} must be non-negative"))
}

fn extend_config_options(
    name: &str,
    value: &toml::Value,
    args: &mut Vec<String>,
) -> Result<(), String> {
    if let Some(value) = value.as_str() {
        extend_config_option_tokens(name, value, args)?;
        return Ok(());
    }
    let Some(values) = value.as_array() else {
        return Err(format!(
            "configuration key {name} must be a string or array of strings"
        ));
    };
    for value in values {
        let Some(option) = value.as_str() else {
            return Err(format!(
                "configuration key {name} must contain only strings"
            ));
        };
        extend_config_option_tokens(name, option, args)?;
    }
    Ok(())
}

fn extend_config_option_tokens(
    name: &str,
    option_text: &str,
    args: &mut Vec<String>,
) -> Result<(), String> {
    let tokens = split_auth_options(option_text)?;
    if tokens
        .iter()
        .any(|token| token == "--config" || token.starts_with("--config="))
    {
        return Err(format!(
            "configuration key {name} must not contain --config; choose the configuration file before loading it"
        ));
    }
    args.extend(tokens);
    Ok(())
}

fn apply_config_environment(variables: &[(String, String)]) {
    for (name, value) in variables {
        set_runtime_env_default(name, value);
    }
}

fn split_auth_options(input: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (None, '\'' | '"') => quote = Some(ch),
            (Some(q), c) if c == q => quote = None,
            (None, c) if c.is_whitespace() => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            (None, '\\') => current.push(ch),
            (Some(_), '\\') => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push('\\');
                }
            }
            (_, c) => current.push(c),
        }
    }

    if quote.is_some() {
        return Err("unterminated quote in AUTH_OPTIONS".to_string());
    }
    if !current.is_empty() {
        out.push(current);
    }
    Ok(out)
}

fn parse_cache_time(value: &str) -> Result<u64, String> {
    let seconds = value
        .parse::<u64>()
        .map_err(|_| format!("invalid --cache-time value {value}; expected 0-120"))?;
    if seconds <= 120 {
        Ok(seconds)
    } else {
        Err(format!(
            "invalid --cache-time value {seconds}; maximum is 120 seconds"
        ))
    }
}

fn parse_color_mode(value: &str) -> Result<ColorMode, String> {
    match value {
        "auto" => Ok(ColorMode::Auto),
        "always" => Ok(ColorMode::Always),
        "never" => Ok(ColorMode::Never),
        other => Err(format!(
            "invalid --color value {other}; expected auto, always, or never"
        )),
    }
}

fn print_help() {
    if !io::stdout().is_terminal() {
        print!("{HELP}");
        return;
    }

    let pager = env::var("PAGER").unwrap_or_else(|_| "less -R".to_string());
    if try_pager(&pager).is_ok() || try_pager("more").is_ok() {
        return;
    }

    print!("{HELP}");
}

fn try_pager(command_line: &str) -> Result<(), String> {
    let parts = split_auth_options(command_line)?;
    let Some((program, args)) = parts.split_first() else {
        return Err("empty pager".to_string());
    };
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(HELP.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("pager failed: {status}"))
    }
}
