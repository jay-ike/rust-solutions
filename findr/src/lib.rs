use std::{collections::HashMap, error::Error, fs, usize};
use regex::Regex;
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
    delete: bool,
    entry_types: Vec<EntryType>,
    max_depth: usize,
    min_depth: usize,
    names: Vec<Regex>,
    paths: Vec<String>,
    size: Option<String>,
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
            Arg::new("max_depth")
            .long("max-depth")
            .value_parser(value_parser!(usize))
            .action(ArgAction::Set)
        )
        .arg(
            Arg::new("delete")
            .long("delete")
            .action(ArgAction::SetTrue)
            .help("delete result entries")
        )
        .arg(
            Arg::new("min_depth")
            .long("min-depth")
            .value_parser(value_parser!(usize))
            .action(ArgAction::Set)
            .default_value("0")
        )
        .arg(
            Arg::new("size")
            .short('s')
            .long("size")
            .allow_hyphen_values(true)
            .value_parser(|v: &str| match parse_size(v) {
                    Some(val) => Ok(val),
                    _ => Err(format!("invalid size: {}", v))

            })
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
        delete: matches.get_flag("delete"),
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
        max_depth: *matches.get_one::<usize>("max_depth").unwrap_or(&usize::MAX),
        min_depth: *matches.get_one::<usize>("min_depth").unwrap(),
        names: matches.get_many::<String>("name")
        .unwrap_or_default()
        .map(|v| {
                match Regex::new(&v.as_str()) {
                    Ok(val) => val,
                    Err(_) => unimplemented!("error: invalid value \'{}\'", &v.as_str())
                }
        })
        .into_iter()
        .collect(),
        paths: matches.get_many::<String>("path")
        .expect("should provide the path")
        .map(|s| s.as_str().to_string())
        .into_iter()
        .collect(),
        size: matches.get_one::<String>("size").cloned()
    })
}



pub fn run(config: Config) -> MyResult<()> {
    let type_filter = |entry: &DirEntry| -> bool{
        config.entry_types.is_empty()
        || config.entry_types.iter()
            .any(|v| v.match_type(&entry.file_type()))
    };
    let name_filter = |entry: &DirEntry| -> bool {
        config.names.is_empty()
        || config.names.iter()
            .any(|re| re.is_match(&entry.file_name().to_str().unwrap()))
    };
    let deletion_filter = |entry: &DirEntry| -> bool {
        if config.delete {
            let file_type = entry.file_type();
            if file_type.is_file() || file_type.is_symlink() {
                fs::remove_file(entry.path().to_str().unwrap()).unwrap();
            } else if file_type.is_dir() {
               fs::remove_dir_all(entry.path().to_str().unwrap()).unwrap();
            }
            return false;
        }
        return true;
    };
    for path in &config.paths {
        let entries = WalkDir::new(path)
            .min_depth(config.min_depth)
            .max_depth(config.max_depth)
            .into_iter()
            .filter_map(|e| match e {
                Err(e) => {
                    eprintln!("{}", e);
                    None
                },
                Ok(entry) => Some(entry)
            })
            .filter(type_filter)
            .filter(name_filter)
            .filter(deletion_filter)
            .filter(|entry| get_size_filter(config.size.clone(), match entry.metadata() {
               Ok(meta) => meta.len().try_into().unwrap(),
                _ => 0
            }))
            .filter(|_| config.max_depth >= config.min_depth)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();
        println!("{}", entries.join("\n"));
    }
    Ok(())
}

pub fn parse_size(size: &str) -> Option<String> {
    let re = Regex::new(r"^(?<sign>[+-])?(?<factor>\d+)(?<unit>[ckMGTP])$")
        .expect("should match pattern");
    if re.is_match(size) {
        return Some(size.to_string());
    }
    None

}

fn get_size_filter(size: Option<String>, file_size: usize) -> bool {
    if size.is_none() {
        return true;
    }
    let val = size.unwrap();
    let re = Regex::new(r"^(?<sign>[+-])?(?<factor>\d+)(?<unit>[ckMGTP])$")
        .expect("should match pattern");
    let caps = re.captures(val.as_str())
        .expect("failed to capture group");
    let factor = caps.name("factor")
        .expect("no factor in size")
        .as_str().parse::<usize>().unwrap();
    let unit: usize = match caps.name("unit") {
        None => 1,
        Some(name) => match name.as_str() {
            "c" => 1,
            "k" => 1024,
            "M" => 1024 * 1024,
            "G" => 1024 * 1024 * 1024,
            "T" => 1024 * 1024 * 1024 * 1024,
            "P" => 1024 * 1024 * 1024 * 1024 * 1024,
            _ => 1
        }
    };
    let request_size: usize = factor * unit;
    match caps.name("sign") {
        None => file_size.eq(&request_size),
        Some(symbol) =>  {
            match symbol.as_str() {
                "+" => file_size.gt(&request_size),
                "-" => file_size.lt(&request_size),
                _ => unreachable!("sign not supported")
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::parse_size;

    #[test]
    fn test_parse_size() {
        assert!(parse_size("foo").is_none());
        assert!(parse_size("+30L").is_none());
        assert!(parse_size("-20k").is_some());
        assert!(parse_size("+20G").is_some());
        assert!(parse_size("20T").is_some());
    }
}
