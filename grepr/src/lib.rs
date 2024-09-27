use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    mem,
};

use clap::{value_parser, Arg, ArgAction, Command};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let pattern;
    let matches = Command::new("grepr")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .version("0.1.0")
        .about("rust implementation of the grep command")
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .help("Count occurences")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("insensitive")
                .short('i')
                .long("insensitive")
                .help("Case-insensitive")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("invert")
                .short('v')
                .long("invert-match")
                .help("Invert match")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("recurse")
                .short('r')
                .long("recursive")
                .help("Recursive search")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pattern")
                .value_name("PATTERN")
                .help("Search pattern")
                .value_parser(value_parser!(String))
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .num_args(1..)
                .value_parser(value_parser!(String))
                .action(ArgAction::Set)
                .default_value("-"),
        )
        .get_matches();
    pattern = matches
        .get_one::<String>("pattern")
        .expect("pattern is required");
    let pattern = RegexBuilder::new(pattern)
        .case_insensitive(matches.get_flag("insensitive"))
        .build()
        .map_err(|_| format!("Invalid pattern \"{}\"", pattern))?;
    Ok(Config {
        pattern,
        files: matches
            .get_many::<String>("files")
            .unwrap_or_default()
            .into_iter()
            .map(String::from)
            .collect(),
        recursive: matches.get_flag("recurse"),
        count: matches.get_flag("count"),
        invert_match: matches.get_flag("invert"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let entries = find_files(&config.files, config.recursive);
    for entry in &entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => match open(&filename) {
                Err(e) => eprintln!("{}: {}", filename, e),
                Ok(file) => {
                    let f = if entries.len() > 1 {
                        format!("{}:", filename)
                    } else {
                        "".to_string()
                    };
                    let matches = find_lines(file, &config.pattern, config.invert_match)?;
                    if config.count {
                        print!("{}{}\n", f, matches.len());
                    } else {
                        matches.iter().for_each(|line| {
                            print!("{}{}", f, line);
                        })
                    }
                }
            },
        }
    }
    Ok(())
}

pub fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn find_lines<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<String>> {
    let mut matches = vec![];
    let mut line = String::new();
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if pattern.is_match(line.as_str()) ^ invert_match {
            matches.push(mem::take(&mut line));
        }
        line.clear();
    }
    Ok(matches)
}

pub fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut files = vec![];
    for path in paths {
        match path.as_str() {
            "-" => files.push(Ok(path.to_string())),
            _ => match std::fs::metadata(path) {
                Err(e) => files.push(Err(format!("{}: {}", path, e).into())),
                Ok(data) => {
                    if data.is_dir() {
                        if recursive {
                            WalkDir::new(path)
                                .sort_by_file_name()
                                .into_iter()
                                .flatten()
                                .filter(|e| e.file_type().is_file())
                                .map(|e| e.path().to_str().unwrap().to_string())
                                .for_each(|s| files.push(Ok(s)));
                        } else {
                            files.push(Err(From::from(format!("{} is a directory", path))));
                        }
                    } else if data.is_file() {
                        files.push(Ok(path.to_string()));
                    }
                }
            },
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;
    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }
        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );
        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";
        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();
        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
