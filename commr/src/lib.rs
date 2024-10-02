use clap::{value_parser, Arg, ArgAction, Command};
use std::error::Error;

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
                .required(true)
                .value_parser(value_parser!(String))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("file2")
                .value_name("FILE2")
                .required(true)
                .value_parser(value_parser!(String))
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
    println!("{:?}", config);
    Ok(())
}
