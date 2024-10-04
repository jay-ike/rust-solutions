use crate::TakeValue::*;
use clap::{value_parser, Arg, ArgAction, Command};
use std::error::Error;
use regex::Regex;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

fn parse_value(value: &str) -> Result<TakeValue, String> {
    let re = Regex::new(r"^(\+)?(\d+)$").unwrap();
    if !re.is_match(value) {
        return Err(format!("illegal value -- {}", value));
    }
    let result = value.parse::<i64>().unwrap();
    if value.starts_with('+') && result == 0 {
        return Ok(PlusZero);
    }
    Ok(TakeNum(if value.starts_with('+') {1 * result} else {-1 * result}))
}
pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("tailr")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .version("0.1.0")
        .about("A Rust implementation of the tail command")
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress headers")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .value_name("BYTES")
                .help("Number of bytes")
                .action(ArgAction::Set)
                .value_parser(parse_value)
                .conflicts_with("lines"),
        )
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .action(ArgAction::Set)
                .value_parser(parse_value)
                .default_value("10"),
        )
        .arg(
            Arg::new("files")
                .value_name("FILES")
                .num_args(1..)
                .action(ArgAction::Set)
                .default_value("-")
                .value_parser(value_parser!(String)),
        )
        .get_matches();
    Ok(Config {
        files: matches.get_many::<String>("files")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect(),
        lines: matches.get_one::<TakeValue>("lines").expect("lines should be provided").clone(),
        bytes: matches.get_one::<TakeValue>("bytes").cloned(),
        quiet: matches.get_flag("quiet"),
    })
}
pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}
