use assert_cmd::Command;
use predicates::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use std::fs;
use::wcr::MyResult;

const PRG: &str = "wcr";
const EMPTY: &str = "tests/inputs/empty.txt";
const FOX: &str = "tests/inputs/fox.txt";
const ATLAMAL: &str = "tests/inputs/atlamal.txt";

// --------------------------------------------------
fn gen_bad_file() -> String {
    loop {
        let filename = rand::thread_rng()
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
fn run(args: &[&str], expected_file: &str) -> MyResult<()> {
    let expected = fs::read_to_string(expected_file)?;
    let output = Command::cargo_bin(PRG)?.args(args).output().expect("fail");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);

    Ok(())
}

// --------------------------------------------------
#[test]
fn skips_bad_file() -> MyResult<()> {
    let bad = gen_bad_file();
    let expected = format!("{bad}: .* [(]os error 2[)]");
    Command::cargo_bin(PRG)?
        .arg(bad)
        .assert()
        .success()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
#[test]
fn empty() -> MyResult<()> {
    run(&[EMPTY], "tests/expected/empty.txt.out")
}

// --------------------------------------------------
#[test]
fn fox() -> MyResult<()> {
    run(&[FOX], "tests/expected/fox.txt.out")
}

// --------------------------------------------------
#[test]
fn fox_bytes() -> MyResult<()> {
    run(&["--bytes", FOX], "tests/expected/fox.txt.c.out")
}

// --------------------------------------------------
#[test]
fn fox_chars() -> MyResult<()> {
    run(&["--chars", FOX], "tests/expected/fox.txt.m.out")
}

// --------------------------------------------------
#[test]
fn fox_words() -> MyResult<()> {
    run(&["--words", FOX], "tests/expected/fox.txt.w.out")
}

// --------------------------------------------------
#[test]
fn fox_lines() -> MyResult<()> {
    run(&["--lines", FOX], "tests/expected/fox.txt.l.out")
}

// --------------------------------------------------
#[test]
fn fox_words_bytes() -> MyResult<()> {
    run(&["-w", "-c", FOX], "tests/expected/fox.txt.wc.out")
}

// --------------------------------------------------
#[test]
fn fox_words_lines() -> MyResult<()> {
    run(&["-w", "-l", FOX], "tests/expected/fox.txt.wl.out")
}

// --------------------------------------------------
#[test]
fn fox_bytes_lines() -> MyResult<()> {
    run(&["-l", "-c", FOX], "tests/expected/fox.txt.cl.out")
}

// --------------------------------------------------
#[test]
fn atlamal() -> MyResult<()> {
    run(&[ATLAMAL], "tests/expected/atlamal.txt.out")
}

// --------------------------------------------------
#[test]
fn atlamal_bytes() -> MyResult<()> {
    run(&["-c", ATLAMAL], "tests/expected/atlamal.txt.c.out")
}

// --------------------------------------------------
#[test]
fn atlamal_words() -> MyResult<()> {
    run(&["-w", ATLAMAL], "tests/expected/atlamal.txt.w.out")
}

// --------------------------------------------------
#[test]
fn atlamal_lines() -> MyResult<()> {
    run(&["-l", ATLAMAL], "tests/expected/atlamal.txt.l.out")
}

// --------------------------------------------------
#[test]
fn atlamal_words_bytes() -> MyResult<()> {
    run(&["-w", "-c", ATLAMAL], "tests/expected/atlamal.txt.wc.out")
}

// --------------------------------------------------
#[test]
fn atlamal_words_lines() -> MyResult<()> {
    run(&["-w", "-l", ATLAMAL], "tests/expected/atlamal.txt.wl.out")
}

// --------------------------------------------------
#[test]
fn atlamal_bytes_lines() -> MyResult<()> {
    run(&["-l", "-c", ATLAMAL], "tests/expected/atlamal.txt.cl.out")
}

// --------------------------------------------------
#[test]
fn atlamal_stdin() -> MyResult<()> {
    let input = fs::read_to_string(ATLAMAL)?;
    let expected =
        fs::read_to_string("tests/expected/atlamal.txt.stdin.out")?;

    let output = Command::cargo_bin(PRG)?
        .write_stdin(input)
        .output()
        .expect("fail");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);
    Ok(())
}

// --------------------------------------------------
#[test]
fn test_all() -> MyResult<()> {
    run(&[EMPTY, FOX, ATLAMAL], "tests/expected/all.out")
}

// --------------------------------------------------
#[test]
fn test_all_lines() -> MyResult<()> {
    run(&["-l", EMPTY, FOX, ATLAMAL], "tests/expected/all.l.out")
}

// --------------------------------------------------
#[test]
fn test_all_words() -> MyResult<()> {
    run(&["-w", EMPTY, FOX, ATLAMAL], "tests/expected/all.w.out")
}

// --------------------------------------------------
#[test]
fn test_all_bytes() -> MyResult<()> {
    run(&["-c", EMPTY, FOX, ATLAMAL], "tests/expected/all.c.out")
}

// --------------------------------------------------
#[test]
fn test_all_words_bytes() -> MyResult<()> {
    run(&["-cw", EMPTY, FOX, ATLAMAL], "tests/expected/all.wc.out")
}

// --------------------------------------------------
#[test]
fn test_all_words_lines() -> MyResult<()> {
    run(&["-wl", EMPTY, FOX, ATLAMAL], "tests/expected/all.wl.out")
}

// --------------------------------------------------
#[test]
fn test_all_bytes_lines() -> MyResult<()> {
    run(&["-cl", EMPTY, FOX, ATLAMAL], "tests/expected/all.cl.out")
}
