use assert_cmd::Command;
use predicates::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use std::{borrow::Cow, fs};
use treer::MyResult;

const PRG: &str = "treer";

fn gen_bad_file() -> String {
    loop {
        let filename: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        if fs::metadata(&filename).is_err() {
            return filename;
        }
    }
}
#[cfg(windows)]
fn format_file_name(file: &str) -> Cow<str> {
    format!("{}.windows", file).into()
}
#[cfg(not(windows))]
fn format_file_name(file: &str) -> Cow<str> {
    file.into()
}
fn run(args: &[&str], expected_file: &str) -> MyResult<()> {
    let file_name = format_file_name(expected_file);
    let contents = fs::read_to_string(file_name.as_ref())?;
    let mut expected: Vec<String> = contents
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.replace("\u{a0}", " "))
        .collect();
    expected.sort();
    let cmd = Command::cargo_bin(PRG)?.args(args).assert().success();
    let stdout = String::from_utf8(cmd.get_output().stdout.clone())?;
    let mut lines: Vec<String> = stdout
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.replace("\u{a0}", " "))
        .collect();
    lines.sort();
    assert_eq!(lines, expected);
    Ok(())
}

#[test]
fn skip_bad_dir() -> MyResult<()> {
    let bad_file = gen_bad_file();
    let expected = format!("{}: .* [(]os error [23][)]", &bad_file);
    Command::cargo_bin(PRG)?
        .arg(bad_file)
        .assert()
        .success()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}
#[test]
fn path1() -> MyResult<()> {
    run(&["tests/inputs"], "tests/expected/path1.txt")
}
#[test]
fn path_a() -> MyResult<()> {
    run(&["tests/inputs/a"], "tests/expected/path_a.txt")
}
#[test]
fn path_a_b() -> MyResult<()> {
    run(&["tests/inputs/a/b"], "tests/expected/path_a_b.txt")
}
#[test]
fn path_a_b_d() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "tests/inputs/d"],
        "tests/expected/path_a_b_d.txt",
    )
}
#[test]
fn depth_2() -> MyResult<()> {
    run(&["tests/inputs", "-L", "2"], "tests/expected/depth_2.txt")
}
#[test]
fn path_a_b_d_depth_2() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "tests/inputs/d", "-L", "2"],
        "tests/expected/path_a_b_d_depth_2.txt",
    )
}
