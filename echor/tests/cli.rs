use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn fail_with_no_args() {
    let mut cmd = Command::cargo_bin("echor").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn runs_ok() {
    let mut cmd = Command::cargo_bin("echor").unwrap();
    cmd.arg("hello")
        .arg("world")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}
