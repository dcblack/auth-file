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
        .stdout(predicate::str::contains("--version"))
        .stdout(predicate::str::contains("--no-platform-auth").not());
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
    let db = tmp.path().join("auth-test");
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    fs::write(&first, "first approved contents\n").unwrap();
    fs::write(&second, "second approved contents\n").unwrap();

    Command::cargo_bin("auth")
        .unwrap()
        .args([
            "--no-platform-auth",
            "--dir",
            path_str(&db),
            "--write",
            path_str(&first),
            path_str(&second),
        ])
        .assert()
        .success();

    assert!(db.join("auth.db").exists());
    assert!(db.join("path-hmac.key").exists());
    assert!(!fs::read_to_string(db.join("auth.db"))
        .unwrap_or_default()
        .contains("first.txt"));
}

#[test]
fn check_two_authorized_one_unauthorized_and_one_missing_file() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("auth-test");
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    let third = tmp.path().join("third-unauthorized.txt");
    let fourth = tmp.path().join("fourth-missing.txt");
    fs::write(&first, "first approved contents\n").unwrap();
    fs::write(&second, "second approved contents\n").unwrap();
    fs::write(&third, "not approved\n").unwrap();

    Command::cargo_bin("auth")
        .unwrap()
        .args([
            "--no-platform-auth",
            "--dir",
            path_str(&db),
            "--write",
            path_str(&first),
            path_str(&second),
        ])
        .assert()
        .success();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&first)])
        .assert()
        .success();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&second)])
        .assert()
        .success();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&third)])
        .assert()
        .failure();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&fourth)])
        .assert()
        .failure();

    Command::cargo_bin("auth")
        .unwrap()
        .args([
            "--dir",
            path_str(&db),
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
    let db = tmp.path().join("auth-test");
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    fs::write(&first, "first approved contents\n").unwrap();
    fs::write(&second, "second approved contents\n").unwrap();

    Command::cargo_bin("auth")
        .unwrap()
        .args([
            "--no-platform-auth",
            "--dir",
            path_str(&db),
            "--write",
            path_str(&first),
            path_str(&second),
        ])
        .assert()
        .success();

    Command::cargo_bin("auth")
        .unwrap()
        .args([
            "--no-platform-auth",
            "--dir",
            path_str(&db),
            "--remove",
            path_str(&first),
        ])
        .assert()
        .success();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&first)])
        .assert()
        .failure();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&second)])
        .assert()
        .success();
}

#[test]
fn write_check_and_detect_change() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("auth-test");
    let file = tmp.path().join("data.txt");
    fs::write(&file, "one\n").unwrap();

    Command::cargo_bin("auth")
        .unwrap()
        .args([
            "--no-platform-auth",
            "--dir",
            path_str(&db),
            "--write",
            path_str(&file),
        ])
        .assert()
        .success();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&file)])
        .assert()
        .success();

    fs::write(&file, "two\n").unwrap();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&file)])
        .assert()
        .failure();
}

#[test]
fn no_platform_auth_requires_explicit_auth_test_directory() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("not-auth-test");
    let file = tmp.path().join("file.txt");
    fs::write(&file, "contents\n").unwrap();

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--no-platform-auth", "--write", path_str(&file)])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires an explicit --dir"));

    Command::cargo_bin("auth")
        .unwrap()
        .args([
            "--no-platform-auth",
            "--dir",
            path_str(&db),
            "--write",
            path_str(&file),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("auth-test"));
}

#[test]
fn auth_options_can_supply_test_directory() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("auth-test");
    let file = tmp.path().join("env-options.txt");
    fs::write(&file, "contents\n").unwrap();
    let auth_options = format!("-d {}", path_str(&db));

    Command::cargo_bin("auth")
        .unwrap()
        .env("AUTH_OPTIONS", auth_options)
        .args(["--no-platform-auth", "--write", path_str(&file)])
        .assert()
        .success()
        .stderr(predicate::str::contains("--no-platform-auth is in effect"));

    Command::cargo_bin("auth")
        .unwrap()
        .args(["--dir", path_str(&db), "--check", path_str(&file)])
        .assert()
        .success();
}

#[test]
fn color_always_colors_errors_and_no_color_disables_auto() {
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("auth-test");
    let missing = tmp.path().join("missing.txt");

    Command::cargo_bin("auth")
        .unwrap()
        .args([
            "--color",
            "always",
            "--dir",
            path_str(&db),
            "--check",
            path_str(&missing),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("\u{1b}[31mError:"));

    Command::cargo_bin("auth")
        .unwrap()
        .env("NO_COLOR", "1")
        .args([
            "--color",
            "auto",
            "--dir",
            path_str(&db),
            "--check",
            path_str(&missing),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"))
        .stderr(predicate::str::contains("\u{1b}[31m").not());
}
