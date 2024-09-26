use std::error::Error;

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
    println!("{:?}", config);
    Ok(())
}

pub fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    paths
        .iter()
        .flat_map(|s| {
            let dir = if recursive {
                WalkDir::new(&s).min_depth(1)
            } else {
                WalkDir::new(&s).min_depth(0).max_depth(0)
            };
            dir.into_iter().map(move |e| match e {
                Err(_) => Err(format!("\"{}\": No such file or directory", &s).into()),
                Ok(entry) => {
                    if entry.file_type().is_dir() && !recursive {
                        return Err(format!("{} is a directory", entry.path().display()).into());
                    }
                    Ok(entry.path().to_str().unwrap_or_default().to_string())
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};
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
}
