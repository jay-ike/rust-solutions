use std::error::Error;
use clap::{value_parser, Arg, ArgAction, Command};


pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn parse_positive_int(value: &str) -> MyResult<usize>{
    match value.parse::<usize>() {
        Ok(val) if val > 0 => Ok(val),
        _ => Err(value.into())
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("headr")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .version("0.1.0")
        .about("Rust implementation of the GNU head command")
        .arg(
            Arg::new("files")
            .num_args(1..)
            .value_parser(value_parser!(String))
            .default_value("-")
            .value_name("FILES")
            .help("input files")
        )
        .arg(
            Arg::new("bytes")
            .value_name("BYTES")
            .short('c')
            .long("bytes")
            .conflicts_with("lines")
            .action(ArgAction::Set)
            .value_parser(|val: &str| match parse_positive_int(val) {
                Ok(v) => Ok(v),
                _ => Err(format!("failed to parse bytes value {}", val))
            })
            .help("number of bytes to print")
        )
        .arg(
            Arg::new("lines")
            .value_name("LINES")
            .short('n')
            .long("lines")
            .default_value("10")
            .value_parser(|val: &str| match parse_positive_int(val) {
                Ok(v) => Ok(v),
                _ => Err(format!("failed to parse lines value {}", val))
            })
            .action(ArgAction::Set)
            .help("number of lines to print")
        ).get_matches();
    Ok(
        Config {
            bytes: matches.get_one::<usize>("bytes").copied(),
            files: matches.get_many::<String>("files")
                .expect("at least one file should be specified")
                .map(|v| v.as_str().to_string())
                .into_iter()
                .collect(),
            lines: *matches.get_one::<usize>("lines")
                .expect("lines count unavailable")
        }
    )
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(config);
    Ok(())
}
