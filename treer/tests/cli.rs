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
#[test]
fn show_dir_only() -> MyResult<()> {
    run(&["tests/inputs", "-d"], "tests/expected/dir_only.txt")
}
#[test]
fn path_a_b_d_dir_only() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "tests/inputs/d", "-d"],
        "tests/expected/path_a_b_d_dir_only.txt",
    )
}
#[test]
fn depth_2_dir_only() -> MyResult<()> {
    run(
        &["tests/inputs", "-d", "-L", "2"],
        "tests/expected/depth_2_dir_only.txt",
    )
}
#[test]
fn path1_show_size() -> MyResult<()> {
    run(
        &["tests/inputs", "-H"],
        "tests/expected/path1_with_size.txt",
    )
}
#[test]
fn path_a_b_d_show_size() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "tests/inputs/d", "-H"],
        "tests/expected/path_a_b_d_show_size.txt",
    )
}
#[test]
fn dir_only_with_size() -> MyResult<()> {
    run(
        &["tests/inputs", "-d", "-H"],
        "tests/expected/dir_only_with_size.txt",
    )
}
#[test]
fn path1_csv_mp3() -> MyResult<()> {
    run(
        &["tests/inputs", "-P", ".*csv", "-P", ".*mp3"],
        "tests/expected/path1_csv_mp3.txt",
    )
}
#[test]
fn die_on_pattern_and_dir_only() -> MyResult<()> {
    let msg = "the argument '--pattern <pattern>...' cannot be \
        used with '--dir-only'";
    Command::cargo_bin(PRG)?
        .args(["-P", "\\*csv", "-d"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(msg));
    Ok(())
}
#[test]
fn die_on_invalid_pattern() -> MyResult<()> {
    Command::cargo_bin(PRG)?
        .args(["-P", "*.csv"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: invalid value '*.csv'"));
    Ok(())
}
#[test]
fn die_on_invalid_size_unit() -> MyResult<()> {
    let msg = "invalid value '+4L' for argument --file-size <file-size>";
    Command::cargo_bin(PRG)?
        .args(["-s", "+4L"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(msg));
    Ok(())
}
#[test]
fn die_on_size_filter_and_dir_only() -> MyResult<()> {
    let msg = "the argument '--file-size <file-size>' cannot be \
        used with '--dir-only'";
    Command::cargo_bin(PRG)?
        .args(["-s", "-8K", "-d"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(msg));
    Ok(())
}
#[test]
fn depth_2_with_size_a() -> MyResult<()> {
    run(
        &["tests/inputs", "-P", ".*tsv", "-H", "-L", "2"],
        "tests/expected/depth_2_tsv_with_size.txt",
    )
}
#[test]
fn path1_with_size_gt_1k() -> MyResult<()> {
    run(
        &["tests/inputs", "-s", "+1K"],
        "tests/expected/path1_with_size_gt_1k.txt",
    )
}
#[test]
fn all_with_size_in_bytes() -> MyResult<()> {
    run(
        &["tests/inputs", "-S"],
        "tests/expected/all_with_size_in_bytes.txt",
    )
}
#[test]
fn all_with_perms_and_byte_size() -> MyResult<()> {
    run(
        &["tests/inputs", "-Sp"],
        "tests/expected/all_with_perms_and_size_bytes.txt",
    )
}
#[test]
fn all_with_perms_and_human_size() -> MyResult<()> {
    run(
        &["tests/inputs", "-Hp"],
        "tests/expected/all_with_perms_and_size_human.txt",
    )
}
#[test]
fn depth_2_lt_1k() -> MyResult<()> {
    run(
        &["tests/inputs", "-s", "-1K", "-L", "2"],
        "tests/expected/path1_depth_2_lt_1k.txt",
    )
}
#[test]
fn path_a_b_d_eq_2_csv() -> MyResult<()> {
    run(
        &[
            "tests/inputs/a/b",
            "tests/inputs/d",
            "-s",
            "2",
            "-P",
            ".*csv",
        ],
        "tests/expected/path_a_b_d_eq_2_csv.txt",
    )
}
