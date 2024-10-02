use crate::Column::*;
use clap::{value_parser, Arg, ArgAction, Command};
use std::cmp::Ordering::*;
use std::{
    error::Error,
    io::{BufRead, BufReader},
};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    delimiter: String,
    file1: String,
    file2: String,
    insensitive: bool,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
}

enum Column<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

impl Column<'_> {
    fn print(self, config: &Config) {
        let mut columns: Vec<&str> = vec![];
        match self {
            Col1(val) => {
                if config.show_col1 {
                    columns.push(val);
                }
            }
            Col2(val) => {
                if config.show_col2 {
                    if config.show_col1 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
            Col3(val) => {
                if config.show_col3 {
                    if config.show_col1 {
                        columns.push("");
                    }
                    if config.show_col2 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
        }
        if ! columns.is_empty() {
            println!("{}", columns.join(config.delimiter.as_str()));
        }
    }
}
pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("commr")
        .version("0.1.0")
        .about("rust implementation of the comm command")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .arg(
            Arg::new("show_col1")
                .short('1')
                .help("suppress printing of column 1")
                .action(ArgAction::SetFalse),
        )
        .arg(
            Arg::new("show_col2")
                .short('2')
                .help("suppress printing of column 2")
                .action(ArgAction::SetFalse),
        )
        .arg(
            Arg::new("show_col3")
                .short('3')
                .help("suppress printing of column 3")
                .action(ArgAction::SetFalse),
        )
        .arg(
            Arg::new("insensitive")
                .short('i')
                .help("Case-insensitive comparison of lines")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("delimiter")
                .short('d')
                .long("output-delimiter")
                .value_name("DELIM")
                .help("Output delimiter")
                .default_value("\t"),
        )
        .arg(
            Arg::new("file1")
                .value_name("FILE1")
                .default_value("-")
                .help("Input file 1")
                .value_parser(value_parser!(String))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("file2")
                .value_name("FILE2")
                .value_parser(value_parser!(String))
                .help("Input file 2")
                .default_value("-")
                .action(ArgAction::Set),
        )
        .get_matches();
    Ok(Config {
        delimiter: matches
            .get_one::<String>("delimiter")
            .expect("delimiter not found")
            .to_string(),
        file1: matches
            .get_one::<String>("file1")
            .expect("file 1 should be present")
            .to_string(),
        file2: matches
            .get_one::<String>("file2")
            .expect("file 2 should be present")
            .to_string(),
        insensitive: matches.get_flag("insensitive"),
        show_col1: matches.get_flag("show_col1"),
        show_col2: matches.get_flag("show_col2"),
        show_col3: matches.get_flag("show_col3"),
    })
}
pub fn run(config: Config) -> MyResult<()> {
    let mut file1;
    let mut file2;
    let case = |line: String| {
        if config.insensitive {
            line.to_lowercase()
        } else {
            line
        }
    };

    if &config.file1 == "-" && &config.file2 == "-" {
        return Err("Both input files cannot be STDIN (\"-\")".into());
    } else {
        file1 = open(&config.file1)?
            .lines()
            .filter_map(Result::ok)
            .map(case);
        file2 = open(&config.file2)?
            .lines()
            .filter_map(Result::ok)
            .map(case);
    }
    let mut line1 = file1.next();
    let mut line2 = file2.next();
    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(val1), Some(val2)) => match val1.cmp(val2) {
                Less => {
                    Col1(val1.as_str()).print(&config);
                    line1 = file1.next();
                }
                Greater => {
                    Col2(val2.as_str()).print(&config);
                    line2 = file2.next();
                }
                Equal => {
                    Col3(val2.as_str()).print(&config);
                    line1 = file1.next();
                    line2 = file2.next();
                }
            },
            (None, Some(val2)) => {
                Col2(val2.as_str()).print(&config);
                line2 = file2.next();
            }
            (Some(val1), None) => {
                Col1(val1.as_str()).print(&config);
                line1 = file1.next();
            }
            _ => (),
        }
    }
    Ok(())
}
fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            std::fs::File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
