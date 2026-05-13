#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use authlib::{auth_report, ActionType, AuthOptions, AuthorizationMode, ColorMode, VERSION};
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

Options
-------

  --help, -h                 Display this text using a pager when interactive
  --version                  Display version
  --dir DIR, -d DIR          Specify database directory [default: ~/.auth]
  --check, -ck               Verify specified files are valid
  --write, -wr               Authorize files; requires platform authorization unless disabled
  --remove, -rm              Remove authorization; requires platform authorization unless disabled
  --verbose, -v              Increase verbosity
  --quiet, -q                Report failures only
  --silent, -s               Silent even with failure, useful in scripts
  --force, -f                Reserved for future non-security confirmation prompts
  --color WHEN               Color output: auto, always, never [default: auto]

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
    if args.is_empty() {
        print_help();
        return Ok(true);
    }
    if args[0] == "--help" || args[0] == "-h" {
        print_help();
        return Ok(true);
    }
    if args[0] == "--version" {
        println!("auth {VERSION}");
        return Ok(true);
    }

    let mut action = ActionType::Check;
    let mut options = AuthOptions::default();
    let mut no_platform_auth_requested = false;
    let mut dir_explicit = false;
    let mut overall_ok = true;
    let mut current_files: Vec<String> = Vec::new();

    let flush = |action: ActionType,
                 files: &mut Vec<String>,
                 options: &AuthOptions,
                 overall_ok: &mut bool| {
        if files.is_empty() {
            return;
        }
        let batch = std::mem::take(files);
        match auth_report(action, batch, options) {
            Ok(report) => *overall_ok &= report.ok(),
            Err(e) => {
                if options.verbose >= 0 {
                    eprintln!("{}", options.colorize_error(&format!("Error: {e}")));
                }
                *overall_ok = false;
            }
        }
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-ck" | "--check" => {
                flush(action, &mut current_files, &options, &mut overall_ok);
                action = ActionType::Check;
            }
            "-wr" | "--write" => {
                flush(action, &mut current_files, &options, &mut overall_ok);
                action = ActionType::Write;
            }
            "-rm" | "--remove" => {
                flush(action, &mut current_files, &options, &mut overall_ok);
                action = ActionType::Remove;
            }
            "-v" | "--verbose" => options.verbose = 1,
            "-q" | "--quiet" => options.verbose = 0,
            "-s" | "--silent" => options.verbose = -1,
            "-f" | "--force" => options.force = true,
            "--no-platform-auth" => {
                options.authorization = AuthorizationMode::None;
                no_platform_auth_requested = true;
            }
            "--color" => {
                i += 1;
                let mode = args
                    .get(i)
                    .ok_or_else(|| "missing value after --color".to_string())?;
                options.color = parse_color_mode(mode)?;
            }
            "-d" | "--dir" => {
                i += 1;
                let dir = args
                    .get(i)
                    .ok_or_else(|| "missing directory after --dir".to_string())?;
                options.db_dir = PathBuf::from(dir);
                dir_explicit = true;
            }
            "--help" | "-h" => return Err("--help must be the first option".to_string()),
            "--version" => return Err("--version must be the first option".to_string()),
            unknown if unknown.starts_with('-') => return Err(format!("unknown option {unknown}")),
            filename => current_files.push(filename.to_string()),
        }
        i += 1;
    }

    if no_platform_auth_requested {
        validate_no_platform_auth_dir(&options.db_dir, dir_explicit)?;
        if options.verbose >= 0 {
            eprintln!("{}", options.colorize_warning("Warning: --no-platform-auth is in effect; authorization prompts are disabled for this test database."));
        }
    }

    flush(action, &mut current_files, &options, &mut overall_ok);
    Ok(overall_ok)
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
            (_, '\\') => {
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

fn validate_no_platform_auth_dir(
    db_dir: &std::path::Path,
    dir_explicit: bool,
) -> Result<(), String> {
    if !dir_explicit {
        return Err(
            "--no-platform-auth requires an explicit --dir/-d option whose basename is auth-test"
                .to_string(),
        );
    }
    let basename = db_dir.file_name().and_then(|s| s.to_str()).ok_or_else(|| {
        "--no-platform-auth requires a database directory basename of auth-test".to_string()
    })?;
    if basename != "auth-test" {
        return Err(format!(
            "--no-platform-auth requires --dir/-d to name a directory ending in auth-test, got {}",
            db_dir.display()
        ));
    }
    Ok(())
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
