#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use authlib::{auth_report, change_fallback_password, ActionType, AuthOptions, ColorMode, VERSION};
use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

const HELP: &str = r#"Name
----

  auth - set, check, or remove authorization for files

Synopsis
--------

  auth --help
  auth --version
  auth --write  [OPTIONS] FILENAME(S)
  auth --check  [OPTIONS] FILENAME(S)
  auth --remove [OPTIONS] FILENAME(S)
  auth --change-password [OPTIONS]

Options
-------

  --help, -h                 Display this text using a pager when interactive
  --version                  Display version
  --dir DIR, -d DIR          Specify database directory [default: ~/.auth]
  --check, -ck               Verify specified files are valid
  --write, -wr               Authorize files; requires platform authorization or auth password
  --remove, -rm              Remove authorization; requires platform authorization or auth password
  --change-password          Change auth password using current password or burner
  --verbose, -v              Increase verbosity
  --quiet, -q                Report failures only
  --silent, -s               Silent even with failure, useful in scripts
  --force, -f                Reserved for future non-security confirmation prompts
  --color WHEN               Color output: auto, always, never [default: auto]
  --cache-time SECONDS       Cache successful authorization for 0-120 seconds [default: 0]

Environment
-----------

  AUTH_OPTIONS               Initial options parsed before command-line arguments.
                             Example: export AUTH_OPTIONS="-d ./auth-test"

  NO_COLOR, NOCOLOR          Disable colored output unless --color always is given.

  PAGER                      Pager used for --help when stdout is a terminal.
                             Defaults to less -R, then more.

Compatibility
-------------

  Options may appear between filenames, allowing a mix of actions in one call.
  AUTH_OPTIONS is processed first, then command-line arguments override or extend it.

  Example:

    auth --write a.txt b.txt
    auth --check a.txt b.txt

Exit Status
-----------

  0  all requested operations succeeded
  1  at least one requested operation failed
  2  command-line usage error
"#;

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
    let args = collect_args()?;
    if handle_top_level_command(&args) {
        return Ok(true);
    }
    execute_args(&args)
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
        println!("auth {VERSION}");
        return true;
    }
    false
}

struct CliState {
    action: ActionType,
    options: AuthOptions,
    overall_ok: bool,
    current_files: Vec<String>,
    change_password_requested: bool,
}

impl Default for CliState {
    fn default() -> Self {
        Self {
            action: ActionType::Check,
            options: AuthOptions::default(),
            overall_ok: true,
            current_files: Vec::new(),
            change_password_requested: false,
        }
    }
}

fn execute_args(args: &[String]) -> Result<bool, String> {
    let mut state = CliState::default();
    let mut i = 0;
    while i < args.len() {
        parse_one_arg(args, &mut i, &mut state)?;
        i += 1;
    }
    finalize_cli_state(&mut state)
}

fn parse_one_arg(args: &[String], i: &mut usize, state: &mut CliState) -> Result<(), String> {
    match args[*i].as_str() {
        "-ck" | "--check" => switch_action(state, ActionType::Check),
        "-wr" | "--write" => switch_action(state, ActionType::Write),
        "-rm" | "--remove" => switch_action(state, ActionType::Remove),
        "--change-password" => state.change_password_requested = true,
        "-v" | "--verbose" => state.options.verbose = 1,
        "-q" | "--quiet" => state.options.verbose = 0,
        "-s" | "--silent" => state.options.verbose = -1,
        "-f" | "--force" => state.options.force = true,
        "--color" => {
            *i += 1;
            let mode = args
                .get(*i)
                .ok_or_else(|| "missing value after --color".to_string())?;
            state.options.color = parse_color_mode(mode)?;
        }
        "--cache-time" => {
            *i += 1;
            let seconds = args
                .get(*i)
                .ok_or_else(|| "missing value after --cache-time".to_string())?;
            state.options.cache_seconds = parse_cache_time(seconds)?;
        }
        "-d" | "--dir" => {
            *i += 1;
            let dir = args
                .get(*i)
                .ok_or_else(|| "missing directory after --dir".to_string())?;
            state.options.db_dir = PathBuf::from(dir);
        }
        "--help" | "-h" => return Err("--help must be the first option".to_string()),
        "--version" => return Err("--version must be the first option".to_string()),
        unknown if unknown.starts_with('-') => return Err(format!("unknown option {unknown}")),
        filename => state.current_files.push(filename.to_string()),
    }
    Ok(())
}

fn switch_action(state: &mut CliState, action: ActionType) {
    flush_current_files(state);
    state.action = action;
}

fn finalize_cli_state(state: &mut CliState) -> Result<bool, String> {
    if state.change_password_requested {
        if !state.current_files.is_empty() {
            return Err("--change-password cannot be mixed with file operands".to_string());
        }
        let burners = change_fallback_password(&state.options).map_err(|e| e.to_string())?;
        eprintln!(
            "{}",
            state.options.colorize_warning(
                "CRITICAL: Save these one-time burner passwords in a password manager. They will not be shown again."
            )
        );
        eprintln!("Database: {}", state.options.db_dir.display());
        for burner in burners {
            eprintln!("  {burner}");
        }
        return Ok(true);
    }

    flush_current_files(state);
    Ok(state.overall_ok)
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

fn collect_args() -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    if let Ok(extra) = env::var("AUTH_OPTIONS") {
        args.extend(split_auth_options(&extra)?);
    }
    args.extend(env::args().skip(1));
    Ok(args)
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
