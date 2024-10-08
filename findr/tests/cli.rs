use assert_cmd::Command;
use predicates::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use std::{borrow::Cow, fs, path::Path};
use findr::MyResult;

const PRG: &str = "findr";

// --------------------------------------------------
fn gen_bad_file() -> String {
    loop {
        let filename: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        if fs::metadata(&filename).is_err() {
            return filename;
        }
    }
}

// --------------------------------------------------
#[test]
fn skips_bad_dir() -> MyResult<()> {
    let bad = gen_bad_file();
    let expected = format!("{}: .* [(]os error [23][)]", &bad);
    Command::cargo_bin(PRG)?
        .arg(&bad)
        .assert()
        .success()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bad_name() -> MyResult<()> {
    Command::cargo_bin(PRG)?
        .args(["--name", "*.csv"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: invalid value '*.csv'"));
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bad_type() -> MyResult<()> {
    let expected = "error: invalid value 'x' for '--type [<TYPE>...]'";
    Command::cargo_bin(PRG)?
        .args(["--type", "x"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
    Ok(())
}

// --------------------------------------------------
#[cfg(windows)]
fn format_file_name(expected_file: &str) -> Cow<str> {
    // Equivalent to: Cow::Owned(format!("{}.windows", expected_file))
    format!("{}.windows", expected_file).into()
}

// --------------------------------------------------
#[cfg(not(windows))]
fn format_file_name(expected_file: &str) -> Cow<str> {
    // Equivalent to: Cow::Borrowed(expected_file)
    expected_file.into()
}

// --------------------------------------------------
fn run(args: &[&str], expected_file: &str) -> MyResult<()> {
    let file = format_file_name(expected_file);
    let contents = fs::read_to_string(file.as_ref())?;
    let mut expected: Vec<&str> =
        contents.split('\n').filter(|s| !s.is_empty()).collect();
    expected.sort();

    let cmd = Command::cargo_bin(PRG)?.args(args).assert().success();
    let out = cmd.get_output();
    let stdout = String::from_utf8(out.stdout.clone())?;
    let mut lines: Vec<&str> =
        stdout.split('\n').filter(|s| !s.is_empty()).collect();
    lines.sort();

    assert_eq!(lines, expected);

    Ok(())
}

// --------------------------------------------------
#[test]
fn path1() -> MyResult<()> {
    run(&["tests/inputs"], "tests/expected/path1.txt")
}

// --------------------------------------------------
#[test]
fn path_a() -> MyResult<()> {
    run(&["tests/inputs/a"], "tests/expected/path_a.txt")
}

// --------------------------------------------------
#[test]
fn path_a_b() -> MyResult<()> {
    run(&["tests/inputs/a/b"], "tests/expected/path_a_b.txt")
}

// --------------------------------------------------
#[test]
fn path_d() -> MyResult<()> {
    run(&["tests/inputs/d"], "tests/expected/path_d.txt")
}

// --------------------------------------------------
#[test]
fn path_a_b_d() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "tests/inputs/d"],
        "tests/expected/path_a_b_d.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_f() -> MyResult<()> {
    run(&["tests/inputs", "-t", "f"], "tests/expected/type_f.txt")
}

// --------------------------------------------------
#[test]
fn type_f_path_a() -> MyResult<()> {
    run(
        &["tests/inputs/a", "-t", "f"],
        "tests/expected/type_f_path_a.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_f_path_a_b() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "--type", "f"],
        "tests/expected/type_f_path_a_b.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_f_path_d() -> MyResult<()> {
    run(
        &["tests/inputs/d", "--type", "f"],
        "tests/expected/type_f_path_d.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_f_path_a_b_d() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "tests/inputs/d", "--type", "f"],
        "tests/expected/type_f_path_a_b_d.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_d() -> MyResult<()> {
    run(&["tests/inputs", "-t", "d"], "tests/expected/type_d.txt")
}

// --------------------------------------------------
#[test]
fn type_d_path_a() -> MyResult<()> {
    run(
        &["tests/inputs/a", "-t", "d"],
        "tests/expected/type_d_path_a.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_d_path_a_b() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "--type", "d"],
        "tests/expected/type_d_path_a_b.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_d_path_d() -> MyResult<()> {
    run(
        &["tests/inputs/d", "--type", "d"],
        "tests/expected/type_d_path_d.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_d_path_a_b_d() -> MyResult<()> {
    run(
        &["tests/inputs/a/b", "tests/inputs/d", "--type", "d"],
        "tests/expected/type_d_path_a_b_d.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_l() -> MyResult<()> {
    run(&["tests/inputs", "-t", "l"], "tests/expected/type_l.txt")
}

// --------------------------------------------------
#[test]
fn type_f_l() -> MyResult<()> {
    run(
        &["tests/inputs", "-t", "l", "f"],
        "tests/expected/type_f_l.txt",
    )
}

// --------------------------------------------------
#[test]
fn name_csv() -> MyResult<()> {
    run(
        &["tests/inputs", "-n", ".*[.]csv"],
        "tests/expected/name_csv.txt",
    )
}

// --------------------------------------------------
#[test]
fn name_csv_mp3() -> MyResult<()> {
    run(
        &["tests/inputs", "-n", ".*[.]csv", "-n", ".*[.]mp3"],
        "tests/expected/name_csv_mp3.txt",
    )
}

// --------------------------------------------------
#[test]
fn name_txt_path_a_d() -> MyResult<()> {
    run(
        &["tests/inputs/a", "tests/inputs/d", "--name", ".*.txt"],
        "tests/expected/name_txt_path_a_d.txt",
    )
}

// --------------------------------------------------
#[test]
fn name_a() -> MyResult<()> {
    run(&["tests/inputs", "-n", "a"], "tests/expected/name_a.txt")
}

// --------------------------------------------------
#[test]
fn type_f_name_a() -> MyResult<()> {
    run(
        &["tests/inputs", "-t", "f", "-n", "a"],
        "tests/expected/type_f_name_a.txt",
    )
}

// --------------------------------------------------
#[test]
fn type_d_name_a() -> MyResult<()> {
    run(
        &["tests/inputs", "--type", "d", "--name", "a"],
        "tests/expected/type_d_name_a.txt",
    )
}

// --------------------------------------------------
#[test]
fn path_g() -> MyResult<()> {
    run(&["tests/inputs/g.csv"], "tests/expected/path_g.txt")
}

// --------------------------------------------------
#[test]
fn type_d_min_depth_2() -> MyResult<()> {
    run(
        &["tests/inputs", "--type", "d", "--min-depth", "2"],
        "tests/expected/type_d_min_depth_2.txt"
    )
}

// --------------------------------------------------
#[test]
fn type_d_max_depth_1() -> MyResult<()> {
    run(
        &["tests/inputs", "--type", "d", "--max-depth", "1"],
        "tests/expected/type_d_max_depth_1.txt"
    )
}

// --------------------------------------------------
#[test]
fn type_d_min_depth_1_max_depth_2() -> MyResult<()> {
    run(
        &["tests/inputs", "--type", "d", "--min-depth", "1", "--max-depth", "2"],
        "tests/expected/type_d_min_depth_1_max_depth_2.txt"
    )
}

// --------------------------------------------------
#[test]
fn type_d_min_depth_2_max_depth_1() -> MyResult<()> {
    run(
        &["tests/inputs", "--type", "d", "--min-depth", "2", "--max-depth", "1"],
        "tests/expected/type_d_min_depth_2_max_depth_1.txt"
    )
}

// --------------------------------------------------
#[test]
fn size_gt_7k_type_f() -> MyResult<()> {
    run(
        &["tests/inputs", "--type", "f", "--size", "+7k"],
        "tests/expected/size_gt_7k_type_f.txt"
    )
}

// --------------------------------------------------
#[test]
fn size_lt_1500() -> MyResult<()> {
    run(
        &["tests/inputs", "--size", "-1500c"],
        "tests/expected/size_lt_1500.txt"
    )
}

// --------------------------------------------------
#[test]
fn size_eq_7443b() -> MyResult<()> {
    run(
        &["tests/inputs", "--size", "7443c"],
        "tests/expected/size_eq_7443b.txt"
    )
}

// --------------------------------------------------
#[test]
fn test_file_deletion() -> MyResult<()>{
   let filename = "tests/inputs/to-be-deleted.txt";
    if !Path::new(filename).exists() {
       fs::write(filename, "")?;
    }
    Command::cargo_bin(PRG)?
        .arg("tests/inputs")
        .arg("-n")
        .arg(".*deleted.txt")
        .arg("--delete")
        .assert()
        .success();
    assert!(!Path::new(filename).exists());
    Ok(())
}

// --------------------------------------------------
#[test]
#[cfg(not(windows))]
fn unreadable_dir() -> MyResult<()> {
    let dirname = "tests/inputs/cant-touch-this";
    if !Path::new(dirname).exists() {
        fs::create_dir(dirname)?;
    }

    std::process::Command::new("chmod")
        .args(["000", dirname])
        .status()
        .expect("failed");

    let cmd = Command::cargo_bin(PRG)?
        .arg("tests/inputs")
        .assert()
        .success();
    fs::remove_dir(dirname)?;

    let out = cmd.get_output();
    let stdout = String::from_utf8(out.stdout.clone())?;
    let lines: Vec<&str> =
        stdout.split('\n').filter(|s| !s.is_empty()).collect();

    assert_eq!(lines.len(), 17);

    let stderr = String::from_utf8(out.stderr.clone())?;
    assert!(stderr.contains("cant-touch-this: Permission denied"));
    Ok(())
}
