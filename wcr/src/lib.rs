use std::error::Error;
use clap::{value_parser, Arg, ArgAction, Command};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    bytes: bool,
    chars: bool,
    lines: bool,
    words: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("wcr")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("Rust version of the wc command")
        .arg(
            Arg::new("files")
            .id("files")
            .num_args(1..)
            .default_value("-")
            .value_name("FILES")
            .value_parser(value_parser!(String))
        )
        .arg(
            Arg::new("words")
            .id("words")
            .short('w')
            .long("words")
            .help("print the word counts")
            .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("lines")
            .id("lines")
            .short('l')
            .long("lines")
            .help("print the line counts")
            .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("bytes")
            .id("bytes")
            .short('c')
            .long("bytes")
            .help("print the byte counts")
            .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("chars")
            .id("chars")
            .short('m')
            .long("chars")
            .help("print the character counts")
            .action(ArgAction::SetTrue)
            .conflicts_with("bytes")

        ).get_matches();
    let mut lines = matches.get_flag("lines");
    let mut words = matches.get_flag("words");
    let chars = matches.get_flag("chars");
    let mut bytes = matches.get_flag("bytes");

    if [lines, words, chars, bytes].iter().all(|&v| !v) {
        lines = true;
        words = true;
        bytes = true;
    }
    Ok(Config {
        files: matches.get_many::<String>("files")
            .expect("provide at least one file")
            .into_iter().map(|s| s.as_str().to_string())
            .collect(),
        bytes,
        chars,
        lines,
        words
    })
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(config);
    Ok(())
}
