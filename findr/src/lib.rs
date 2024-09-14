use core::panic;
use std::{collections::HashMap, error::Error, os::unix::ffi::OsStrExt};
use regex::bytes::Regex;
use clap::{value_parser, Arg, ArgAction, Command};
use walkdir::{WalkDir, DirEntry};
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

impl EntryType {
    fn match_type(&self, file_type: &std::fs::FileType) -> bool {
       match self {
            Dir => file_type.is_dir(),
            File => file_type.is_file(),
            Link => file_type.is_symlink(),
       }
    }
}
impl Config {
   fn print(&self, entry: &DirEntry) {
        let good_entry: bool;
        let good_name: bool;
        good_entry = self.entry_types.iter().any(
            |v| v.match_type(&entry.file_type())
        ) || self.entry_types.is_empty();
        good_name = self.names.iter().any(
            |v| v.is_match(&entry.file_name().as_bytes())
        ) || self.names.is_empty();

        if good_name && good_entry {
            println!("{}", entry.path().display());
        }
   }
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
            .action(ArgAction::Append)
        )
        .arg(
            Arg::new("type")
            .num_args(0..)
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
        .unwrap_or_default()
        .fold(HashMap::<String, &str>::new(), |mut acc, v| {
                acc.insert(v.to_string(), v);
                acc
        })
        .values()
        .map(|v| match *v {
                "f" => File,
                "d" => Dir,
                "l" => Link,
                _ => unreachable!("Invalid type")
        })
        .into_iter()
        .collect(),
        names: matches.get_many::<String>("name")
        .unwrap_or_default()
        .map(|v| {
                match Regex::new(&v.as_str()) {
                    Ok(val) => val,
                    Err(_) => panic!("error: invalid value \'{}\'", &v.as_str())
                }
        })
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
    for path in &config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}: ", e),
                Ok(val) => {
                    config.print(&val);
                }
            }
        }
    }
    Ok(())
}
