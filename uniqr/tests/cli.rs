use assert_cmd::Command;
use predicates::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use std::fs;
use tempfile::NamedTempFile;
use uniqr::MyResult;

struct Test {
    input: &'static str,
    out: &'static str,
    out_count: &'static str,
}

const PRG: &str = "uniqr";

const EMPTY: Test = Test {
    input: "tests/inputs/empty.txt",
    out: "tests/expected/empty.txt.out",
    out_count: "tests/expected/empty.txt.c.out",
};

const ONE: Test = Test {
    input: "tests/inputs/one.txt",
    out: "tests/expected/one.txt.out",
    out_count: "tests/expected/one.txt.c.out",
};

const TWO: Test = Test {
    input: "tests/inputs/two.txt",
    out: "tests/expected/two.txt.out",
    out_count: "tests/expected/two.txt.c.out",
};

const THREE: Test = Test {
    input: "tests/inputs/three.txt",
    out: "tests/expected/three.txt.out",
    out_count: "tests/expected/three.txt.c.out",
};

const SKIP: Test = Test {
    input: "tests/inputs/skip.txt",
    out: "tests/expected/skip.txt.out",
    out_count: "tests/expected/skip.txt.c.out",
};

const T1: Test = Test {
    input: "tests/inputs/t1.txt",
    out: "tests/expected/t1.txt.out",
    out_count: "tests/expected/t1.txt.c.out",
};

const T2: Test = Test {
    input: "tests/inputs/t2.txt",
    out: "tests/expected/t2.txt.out",
    out_count: "tests/expected/t2.txt.c.out",
};

const T3: Test = Test {
    input: "tests/inputs/t3.txt",
    out: "tests/expected/t3.txt.out",
    out_count: "tests/expected/t3.txt.c.out",
};

const T4: Test = Test {
    input: "tests/inputs/t4.txt",
    out: "tests/expected/t4.txt.out",
    out_count: "tests/expected/t4.txt.c.out",
};

const T5: Test = Test {
    input: "tests/inputs/t5.txt",
    out: "tests/expected/t5.txt.out",
    out_count: "tests/expected/t5.txt.c.out",
};

const T6: Test = Test {
    input: "tests/inputs/t6.txt",
    out: "tests/expected/t6.txt.out",
    out_count: "tests/expected/t6.txt.c.out",
};

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
fn dies_bad_file() -> MyResult<()> {
    let bad = gen_bad_file();
    let expected = format!("{bad}: .* [(]os error 2[)]");
    Command::cargo_bin(PRG)?
        .arg(bad)
        .assert()
        .failure()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
// HELPER FUNCTIONS
fn run(test: &Test) -> MyResult<()> {
    let expected = fs::read_to_string(test.out)?;
    let output = Command::cargo_bin(PRG)?
        .arg(test.input)
        .output()
        .expect("fail");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);
    Ok(())
}

// --------------------------------------------------
fn run_count(test: &Test) -> MyResult<()> {
    let expected = fs::read_to_string(test.out_count)?;
    let output = Command::cargo_bin(PRG)?
        .args([test.input, "-c"])
        .output()
        .expect("fail");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);
    Ok(())
}

// --------------------------------------------------
fn run_stdin(test: &Test) -> MyResult<()> {
    let input = fs::read_to_string(test.input)?;
    let expected = fs::read_to_string(test.out)?;
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
fn run_stdin_count(test: &Test) -> MyResult<()> {
    let input = fs::read_to_string(test.input)?;
    let expected = fs::read_to_string(test.out_count)?;
    let output = Command::cargo_bin(PRG)?
        .arg("--count")
        .write_stdin(input)
        .output()
        .expect("fail");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);
    Ok(())
}

// --------------------------------------------------
fn run_outfile(test: &Test) -> MyResult<()> {
    let expected = fs::read_to_string(test.out)?;
    let outfile = NamedTempFile::new()?;
    let outpath = &outfile.path().to_str().unwrap();

    Command::cargo_bin(PRG)?
        .args([test.input, outpath])
        .assert()
        .success()
        .stdout("");
    let contents = fs::read_to_string(outpath)?;
    assert_eq!(&expected, &contents);

    Ok(())
}

// --------------------------------------------------
fn run_outfile_count(test: &Test) -> MyResult<()> {
    let outfile = NamedTempFile::new()?;
    let outpath = &outfile.path().to_str().unwrap();

    Command::cargo_bin(PRG)?
        .args([test.input, outpath, "--count"])
        .assert()
        .success()
        .stdout("");

    let expected = fs::read_to_string(test.out_count)?;
    let contents = fs::read_to_string(outpath)?;
    assert_eq!(&expected, &contents);

    Ok(())
}

// --------------------------------------------------
fn run_stdin_outfile_count(test: &Test) -> MyResult<()> {
    let input = fs::read_to_string(test.input)?;
    let outfile = NamedTempFile::new()?;
    let outpath = &outfile.path().to_str().unwrap();

    Command::cargo_bin(PRG)?
        .args(["-", outpath, "-c"])
        .write_stdin(input)
        .assert()
        .stdout("");

    let expected = fs::read_to_string(test.out_count)?;
    let contents = fs::read_to_string(outpath)?;
    assert_eq!(&expected, &contents);

    Ok(())
}

// --------------------------------------------------
#[test]
fn empty() -> MyResult<()> {
    run(&EMPTY)
}

#[test]
fn empty_count() -> MyResult<()> {
    run_count(&EMPTY)
}

#[test]
fn empty_stdin() -> MyResult<()> {
    run_stdin(&EMPTY)
}

#[test]
fn empty_stdin_count() -> MyResult<()> {
    run_stdin_count(&EMPTY)
}

#[test]
fn empty_outfile() -> MyResult<()> {
    run_outfile(&EMPTY)
}

#[test]
fn empty_outfile_count() -> MyResult<()> {
    run_outfile_count(&EMPTY)
}

#[test]
fn empty_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&EMPTY)
}

// --------------------------------------------------
#[test]
fn one() -> MyResult<()> {
    run(&ONE)
}

#[test]
fn one_count() -> MyResult<()> {
    run_count(&ONE)
}

#[test]
fn one_stdin() -> MyResult<()> {
    run_stdin(&ONE)
}

#[test]
fn one_stdin_count() -> MyResult<()> {
    run_stdin_count(&ONE)
}

#[test]
fn one_outfile() -> MyResult<()> {
    run_outfile(&ONE)
}

#[test]
fn one_outfile_count() -> MyResult<()> {
    run_outfile_count(&ONE)
}

#[test]
fn one_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&ONE)
}

// --------------------------------------------------
#[test]
fn two() -> MyResult<()> {
    run(&TWO)
}

#[test]
fn two_count() -> MyResult<()> {
    run_count(&TWO)
}

#[test]
fn two_stdin() -> MyResult<()> {
    run_stdin(&TWO)
}

#[test]
fn two_stdin_count() -> MyResult<()> {
    run_stdin_count(&TWO)
}

#[test]
fn two_outfile() -> MyResult<()> {
    run_outfile(&TWO)
}

#[test]
fn two_outfile_count() -> MyResult<()> {
    run_outfile_count(&TWO)
}

#[test]
fn two_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&TWO)
}

// --------------------------------------------------
#[test]
fn three() -> MyResult<()> {
    run(&THREE)
}

#[test]
fn three_count() -> MyResult<()> {
    run_count(&THREE)
}

#[test]
fn three_stdin() -> MyResult<()> {
    run_stdin(&THREE)
}

#[test]
fn three_stdin_count() -> MyResult<()> {
    run_stdin_count(&THREE)
}

#[test]
fn three_outfile() -> MyResult<()> {
    run_outfile(&THREE)
}

#[test]
fn three_outfile_count() -> MyResult<()> {
    run_outfile_count(&THREE)
}

#[test]
fn three_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&THREE)
}

// --------------------------------------------------
#[test]
fn skip() -> MyResult<()> {
    run(&SKIP)
}

#[test]
fn skip_count() -> MyResult<()> {
    run_count(&SKIP)
}

#[test]
fn skip_stdin() -> MyResult<()> {
    run_stdin(&SKIP)
}

#[test]
fn skip_stdin_count() -> MyResult<()> {
    run_stdin_count(&SKIP)
}

#[test]
fn skip_outfile() -> MyResult<()> {
    run_outfile(&SKIP)
}

#[test]
fn skip_outfile_count() -> MyResult<()> {
    run_outfile_count(&SKIP)
}

#[test]
fn skip_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&SKIP)
}

// --------------------------------------------------
#[test]
fn t1() -> MyResult<()> {
    run(&T1)
}

#[test]
fn t1_count() -> MyResult<()> {
    run_count(&T1)
}

#[test]
fn t1_stdin() -> MyResult<()> {
    run_stdin(&T1)
}

#[test]
fn t1_stdin_count() -> MyResult<()> {
    run_stdin_count(&T1)
}

#[test]
fn t1_outfile() -> MyResult<()> {
    run_outfile(&T1)
}

#[test]
fn t1_outfile_count() -> MyResult<()> {
    run_outfile_count(&T1)
}

#[test]
fn t1_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&T1)
}

// --------------------------------------------------
#[test]
fn t2() -> MyResult<()> {
    run(&T2)
}

#[test]
fn t2_count() -> MyResult<()> {
    run_count(&T2)
}

#[test]
fn t2_stdin() -> MyResult<()> {
    run_stdin(&T2)
}

#[test]
fn t2_stdin_count() -> MyResult<()> {
    run_stdin_count(&T2)
}

#[test]
fn t2_outfile() -> MyResult<()> {
    run_outfile(&T2)
}

#[test]
fn t2_outfile_count() -> MyResult<()> {
    run_outfile_count(&T2)
}

#[test]
fn t2_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&T2)
}

// --------------------------------------------------
#[test]
fn t3() -> MyResult<()> {
    run(&T3)
}

#[test]
fn t3_count() -> MyResult<()> {
    run_count(&T3)
}

#[test]
fn t3_stdin() -> MyResult<()> {
    run_stdin(&T3)
}

#[test]
fn t3_stdin_count() -> MyResult<()> {
    run_stdin_count(&T3)
}

#[test]
fn t3_outfile() -> MyResult<()> {
    run_outfile(&T3)
}

#[test]
fn t3_outfile_count() -> MyResult<()> {
    run_outfile_count(&T3)
}

#[test]
fn t3_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&T3)
}

// --------------------------------------------------
#[test]
fn t4() -> MyResult<()> {
    run(&T4)
}

#[test]
fn t4_count() -> MyResult<()> {
    run_count(&T4)
}

#[test]
fn t4_stdin() -> MyResult<()> {
    run_stdin(&T4)
}

#[test]
fn t4_stdin_count() -> MyResult<()> {
    run_stdin_count(&T4)
}

#[test]
fn t4_outfile() -> MyResult<()> {
    run_outfile(&T4)
}

#[test]
fn t4_outfile_count() -> MyResult<()> {
    run_outfile_count(&T4)
}

#[test]
fn t4_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&T4)
}

// --------------------------------------------------
#[test]
fn t5() -> MyResult<()> {
    run(&T5)
}

#[test]
fn t5_count() -> MyResult<()> {
    run_count(&T5)
}

#[test]
fn t5_stdin() -> MyResult<()> {
    run_stdin(&T5)
}

#[test]
fn t5_stdin_count() -> MyResult<()> {
    run_stdin_count(&T5)
}

#[test]
fn t5_outfile() -> MyResult<()> {
    run_outfile(&T5)
}

#[test]
fn t5_outfile_count() -> MyResult<()> {
    run_outfile_count(&T5)
}

#[test]
fn t5_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&T5)
}

// --------------------------------------------------
#[test]
fn t6() -> MyResult<()> {
    run(&T6)
}

#[test]
fn t6_count() -> MyResult<()> {
    run_count(&T6)
}

#[test]
fn t6_stdin() -> MyResult<()> {
    run_stdin(&T6)
}

#[test]
fn t6_stdin_count() -> MyResult<()> {
    run_stdin_count(&T6)
}

#[test]
fn t6_outfile() -> MyResult<()> {
    run_outfile(&T6)
}

#[test]
fn t6_outfile_count() -> MyResult<()> {
    run_outfile_count(&T6)
}

#[test]
fn t6_stdin_outfile_count() -> MyResult<()> {
    run_stdin_outfile_count(&T6)
}
