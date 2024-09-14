use std::error::Error;
use regex::Regex;
use clap::{parser::ValuesRef, value_parser, Arg, ArgAction, Command};
use crate::EntryType::*;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq)]
enum EntryType {
    Dir,
    File,
    Link
}

#[derive(Debug)]
pub struct Config {
    entry_types: Vec<EntryType>,
    names: Vec<Regex>,
    paths: Vec<String>,
}

pub fn get_args() -> MyResult<Config>{
    let matches = Command::new("findr")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("Rust implementation of the find command")
        .arg(
            Arg::new("name")
            .num_args(0..)
            .value_parser(value_parser!(String))
            .short('n')
            .long("name")
            .value_name("NAME")
            .action(ArgAction::Set)
        )
        .arg(
            Arg::new("type")
            .num_args(0..3)
            .value_parser(["f", "d", "l"])
            .short('t')
            .long("type")
            .value_name("TYPE")
            .action(ArgAction::Set)
        )
        .arg(
            Arg::new("path")
            .num_args(1..)
            .value_parser(value_parser!(String))
            .value_name("PATH")
            .action(ArgAction::Set)
            .default_value(".")
        )
        .get_matches();
    Ok(Config {
        entry_types: matches
        .get_many::<String>("type")
        .unwrap_or(ValuesRef::default())
        .map(|v| match v.as_str() {
                "f" => File,
                "d" => Dir,
                _ => Link
        })
        .into_iter()
        .collect(),
        names: matches.get_many::<String>("name")
        .unwrap_or(ValuesRef::default())
        .map(|v| Regex::new(&v.as_str()).unwrap())
        .into_iter()
        .collect(),
        paths: matches.get_many::<String>("path")
        .expect("should provide the path")
        .map(|s| s.as_str().to_string())
        .into_iter()
        .collect()
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}
