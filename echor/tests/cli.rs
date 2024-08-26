use assert_cmd::Command;
use predicates::prelude::predicate;

type  TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn fail_with_no_args() -> TestResult {
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
    Ok(())
}

#[test]
fn runs_ok() -> TestResult {
    let mut cmd = Command::cargo_bin("echor")?;
    cmd.arg("hello")
        .arg("world")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
    Ok(())
}
