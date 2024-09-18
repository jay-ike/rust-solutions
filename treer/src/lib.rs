use clap::{value_parser, Arg, ArgAction, Command};
use std::{error::Error, fs};
use walkdir::WalkDir;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("treer")
        .about("rust implementation of the tree command")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .version("0.1.0")
        .arg(
            Arg::new("dir")
                .num_args(1..)
                .value_name("DIR")
                .value_parser(value_parser!(String))
                .help("directories to print the tree")
                .action(ArgAction::Set),
        )
        .get_matches();
    Ok(Config {
        paths: matches
            .get_many::<String>("dir")
            .unwrap_or_default()
            .map(|v| v.to_string())
            .collect(),
    })
}
pub fn run(config: Config) -> MyResult<()> {
    let mut total_dirs: usize = 0;
    let mut total_files: usize = 0;
    for path in &config.paths {
        println!("{}", path);
        let (dirs, files) = visit_dir(path, 1, true, "".to_string());
        total_dirs += dirs;
        total_files += files;
    }
    println!("\n{} directories, {} files", total_dirs, total_files);
    Ok(())
}

fn visit_dir(path: &str, depth: usize, ancestor_end: bool, ancestor_bar: String) -> (usize, usize) {
    let mut files: usize = 0;
    let mut dirs: usize = 1;
    let mut entries = WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| match e {
            Err(e) => {
                eprintln!("{}", e);
                None
            }
            Ok(entry) => Some(entry),
        })
        .peekable();
    while let Some(entry) = entries.next() {
        let name = entry.file_name().to_str().unwrap();
        let is_end = entries.peek().is_none();
        let sym = if is_end { "└──" } else { "├──" };
        if entry.file_type().is_dir() {
            let next_bar = format!(
                "{}{:<s$}",
                ancestor_bar,
                if is_end { "" } else { "│" },
                s = 4
            );
            println!("{}{} {}", ancestor_bar, sym, name);
            let (next_dir, next_file) = visit_dir(
                entry.path().to_str().unwrap_or_default(),
                depth + 1,
                ancestor_end && is_end,
                next_bar,
            );
            dirs += next_dir;
            files += next_file;
        } else if entry.file_type().is_file() {
            println!("{}{} {}", ancestor_bar, sym, name);
            files += 1;
        } else if entry.file_type().is_symlink() {
            println!(
                "{}{} {}",
                ancestor_bar,
                sym,
                format!(
                    "{} -> {}",
                    name,
                    fs::read_link(entry.path())
                        .unwrap_or_default()
                        .to_str()
                        .unwrap()
                )
            );
            files += 1;
        }
    }
    (dirs, files)
}
