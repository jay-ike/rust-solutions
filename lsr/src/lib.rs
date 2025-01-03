use clap::{Arg, ArgAction, Command};
use std::error::Error;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("lsr")
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .help("Show all files")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("long")
                .short('l')
                .long("long")
                .help("Long listing")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("paths")
                .value_name("PATHS")
                .default_value(".")
                .help("Files and/or directories")
                .action(ArgAction::Append),
        )
        .get_matches();
    Ok(Config {
        paths: matches
            .get_many::<String>("paths")
            .expect("paths is required")
            .into_iter()
            .map(|v| v.to_string())
            .collect(),
        long: matches.get_flag("long"),
        show_hidden: matches.get_flag("all")
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}
