use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn path_str(path: &Path) -> &str {
    path.to_str().expect("test paths are valid UTF-8")
}

#[test]
fn help_option_works() {
    let mut cmd = Command::cargo_bin("auth").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth --write"))
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn version_option_works() {
    let mut cmd = Command::cargo_bin("auth").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth "));
}

#[test]
fn write_authorization_for_two_files() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("db");
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    fs::write(&first, "first approved contents\n").unwrap();
    fs::write(&second, "second approved contents\n").unwrap();

    Command::cargo_bin("auth").unwrap()
        .args([
            "--no-platform-auth",
            "--dir", path_str(&db),
            "--write",
            path_str(&first),
            path_str(&second),
        ])
        .assert()
        .success();

    let records = fs::read_dir(&db)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("json"))
        .count();
    assert_eq!(records, 2);
}

#[test]
fn check_two_authorized_one_unauthorized_and_one_missing_file() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("db");
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    let third = tmp.path().join("third-unauthorized.txt");
    let fourth = tmp.path().join("fourth-missing.txt");
    fs::write(&first, "first approved contents\n").unwrap();
    fs::write(&second, "second approved contents\n").unwrap();
    fs::write(&third, "not approved\n").unwrap();

    Command::cargo_bin("auth").unwrap()
        .args([
            "--no-platform-auth",
            "--dir", path_str(&db),
            "--write",
            path_str(&first),
            path_str(&second),
        ])
        .assert()
        .success();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&first)])
        .assert()
        .success();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&second)])
        .assert()
        .success();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&third)])
        .assert()
        .failure();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&fourth)])
        .assert()
        .failure();

    Command::cargo_bin("auth").unwrap()
        .args([
            "--dir", path_str(&db),
            "--check",
            path_str(&first),
            path_str(&second),
            path_str(&third),
            path_str(&fourth),
        ])
        .assert()
        .failure();
}

#[test]
fn remove_one_authorized_file_then_check_removed_file_fails() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("db");
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    fs::write(&first, "first approved contents\n").unwrap();
    fs::write(&second, "second approved contents\n").unwrap();

    Command::cargo_bin("auth").unwrap()
        .args([
            "--no-platform-auth",
            "--dir", path_str(&db),
            "--write",
            path_str(&first),
            path_str(&second),
        ])
        .assert()
        .success();

    Command::cargo_bin("auth").unwrap()
        .args([
            "--no-platform-auth",
            "--dir", path_str(&db),
            "--remove",
            path_str(&first),
        ])
        .assert()
        .success();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&first)])
        .assert()
        .failure();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&second)])
        .assert()
        .success();
}

#[test]
fn write_check_and_detect_change() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("db");
    let file = tmp.path().join("data.txt");
    fs::write(&file, "one\n").unwrap();

    Command::cargo_bin("auth").unwrap()
        .args(["--no-platform-auth", "--dir", path_str(&db), "--write", path_str(&file)])
        .assert()
        .success();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&file)])
        .assert()
        .success();

    fs::write(&file, "two\n").unwrap();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&file)])
        .assert()
        .failure();
}
