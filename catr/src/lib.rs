use std::error::Error;
use clap::{value_parser, Arg, ArgAction, Command};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(config);
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches =  Command::new("catr")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("Rust implementation of the cat command")
        .arg(
            Arg::new("files")
            .value_name("FILES")
            .num_args(1..)
            .help("input files")
            .default_value("-")
            .value_parser(value_parser!(String))
        )
        .arg(
            Arg::new("show_blanks")
            .short('b')
            .long("blanks")
            .action(ArgAction::SetFalse)
            .help("Print non-blank lines")
        )
        .arg(
            Arg::new("show_line_number")
            .short('n')
            .long("number")
            .action(ArgAction::SetTrue)
            .help("Print line numbers")
        ).get_matches();
    Ok(Config {
        files: matches.get_many::<String>("files")
            .expect("files should be specified")
            .into_iter()
            .map(|s| s.as_str().to_string())
            .collect(),
        number_lines: matches.get_flag("show_line_number"),
        number_nonblank_lines: matches.get_flag("show_blanks")
    })
}
