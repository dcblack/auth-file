use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn version_option_works() {
    let mut cmd = Command::cargo_bin("auth").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth "));
}

#[test]
fn help_option_works() {
    let mut cmd = Command::cargo_bin("auth").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth --write"));
}

#[test]
fn write_check_and_detect_change() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("db");
    let file = tmp.path().join("data.txt");
    fs::write(&file, "one\n").unwrap();

    Command::cargo_bin("auth").unwrap()
        .args(["--no-platform-auth", "--dir", db.to_str().unwrap(), "--write", file.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", db.to_str().unwrap(), "--check", file.to_str().unwrap()])
        .assert()
        .success();

    fs::write(&file, "two\n").unwrap();

    Command::cargo_bin("auth").unwrap()
        .args(["--dir", db.to_str().unwrap(), "--check", file.to_str().unwrap()])
        .assert()
        .failure();
}
