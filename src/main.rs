#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use authlib::{auth_report, ActionType, AuthOptions, AuthorizationMode, VERSION};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

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

  --help, -h                 Display this text
  --version                  Display version
  --dir DIR, -d DIR          Specify database directory [default: ~/.auth]
  --check, -ck               Verify specified files are valid
  --write, -wr               Authorize files; requires platform authorization unless disabled
  --remove, -rm              Remove authorization; requires platform authorization unless disabled
  --verbose, -v              Increase verbosity
  --quiet, -q                Report failures only
  --silent, -s               Silent even with failure, useful in scripts
  --force, -f                No confirmation/platform authorization required
  --no-platform-auth         Development/CI mode; do not prompt Touch ID/Hello/PAM

Compatibility
-------------

  Options may appear between filenames, allowing a mix of actions in one call.
  Example:

    auth --write a.txt b.txt --check c.txt --remove d.txt

Exit Status
-----------

  0  all requested operations succeeded
  1  at least one requested operation failed
  2  command-line usage error
"#;

fn main() -> ExitCode {
    match run() {
        Ok(ok) => if ok { ExitCode::SUCCESS } else { ExitCode::from(1) },
        Err(msg) => {
            eprintln!("Error: {msg}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        println!("{HELP}");
        return Ok(true);
    }
    if args[0] == "--help" || args[0] == "-h" {
        println!("{HELP}");
        return Ok(true);
    }
    if args[0] == "--version" {
        println!("auth {VERSION}");
        return Ok(true);
    }

    let mut action = ActionType::Check;
    let mut options = AuthOptions::default();
    let mut overall_ok = true;
    let mut current_files: Vec<String> = Vec::new();

    let flush = |action: ActionType, files: &mut Vec<String>, options: &AuthOptions, overall_ok: &mut bool| {
        if files.is_empty() { return; }
        let batch = std::mem::take(files);
        match auth_report(action, batch, options.clone()) {
            Ok(report) => *overall_ok &= report.ok(),
            Err(e) => {
                if options.verbose >= 0 { eprintln!("Error: {e}"); }
                *overall_ok = false;
            }
        }
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-ck" | "--check" => { flush(action, &mut current_files, &options, &mut overall_ok); action = ActionType::Check; }
            "-wr" | "--write" => { flush(action, &mut current_files, &options, &mut overall_ok); action = ActionType::Write; }
            "-rm" | "--remove" => { flush(action, &mut current_files, &options, &mut overall_ok); action = ActionType::Remove; }
            "-v" | "--verbose" => options.verbose = 1,
            "-q" | "--quiet" => options.verbose = 0,
            "-s" | "--silent" => options.verbose = -1,
            "-f" | "--force" => options.force = true,
            "--no-platform-auth" => options.authorization = AuthorizationMode::None,
            "-d" | "--dir" => {
                i += 1;
                let dir = args.get(i).ok_or_else(|| "missing directory after --dir".to_string())?;
                options.db_dir = PathBuf::from(dir);
            }
            "--help" | "-h" => return Err("--help must be the first option".to_string()),
            "--version" => return Err("--version must be the first option".to_string()),
            unknown if unknown.starts_with('-') => return Err(format!("unknown option {unknown}")),
            filename => current_files.push(filename.to_string()),
        }
        i += 1;
    }
    flush(action, &mut current_files, &options, &mut overall_ok);
    Ok(overall_ok)
}
