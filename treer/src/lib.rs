use clap::{value_parser, Arg, ArgAction, Command};
use std::{error::Error, fs};
use walkdir::{DirEntry, WalkDir};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    depth: Option<usize>,
    dir_only: bool,
    paths: Vec<String>,
    show_size: bool,
}

#[derive(Debug, Eq, PartialEq)]
enum SizeUnit {
    K,
    M,
    G,
    T,
    P,
}

impl SizeUnit {
    fn next_unit(&self) -> SizeUnit {
        match self {
            SizeUnit::K => SizeUnit::M,
            SizeUnit::M => SizeUnit::G,
            SizeUnit::G => SizeUnit::T,
            SizeUnit::T => SizeUnit::P,
            SizeUnit::P => SizeUnit::P,
        }
    }
}
impl ToString for SizeUnit {
   fn to_string(&self) -> String {
        match self {
            SizeUnit::K => "K".to_string(),
            SizeUnit::M => "M".to_string(),
            SizeUnit::G => "G".to_string(),
            SizeUnit::T => "T".to_string(),
            SizeUnit::P => "P".to_string(),
        }
   }
}

impl Config {
    pub fn hint_size(&self) -> bool {
        self.show_size
    }
    pub fn path_size(&self, path: &str) -> String {
        if let Ok(meta) = fs::metadata(path) {
            return self.get_printable_size(meta.len());
        }
        "".to_string()
    }
    pub fn entry_size(&self, entry: &DirEntry) -> String {
        let size = entry.metadata().unwrap().len();
        self.get_printable_size(size)
    }
    fn get_printable_size(&self, val: u64) -> String {
        if self.hint_size() {
            format!("[{:>4}]  ", get_file_size(val))
        } else {
            "".to_string()
        }
    }
}

fn get_file_size(size: u64) -> String {
    if size < 1024 {
        return size.to_string();
    }
    evaluate_size(size as f64, SizeUnit::K)
}

fn evaluate_size(size: f64, unit: SizeUnit) -> String {
    let remainder = size / 1024.0;
    if remainder <= 1024.0 || unit == unit.next_unit() {
        return format!("{:.1}{}", remainder, unit.to_string());
    }
    evaluate_size(remainder, unit.next_unit())
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
        .arg(
            Arg::new("depth")
                .value_name("LEVEL")
                .value_parser(value_parser!(usize))
                .help("maximum depth level of the tree")
                .short('L')
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("hint_size")
                .help("show entries size")
                .long("hint-size")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dir_only")
                .help("show only directories")
                .short('d')
                .action(ArgAction::SetTrue),
        )
        .get_matches();
    Ok(Config {
        depth: matches.get_one::<usize>("depth").copied(),
        dir_only: matches.get_flag("dir_only"),
        paths: matches
            .get_many::<String>("dir")
            .unwrap_or_default()
            .map(|v| v.to_string())
            .collect(),
        show_size: matches.get_flag("hint_size"),
    })
}
pub fn run(config: Config) -> MyResult<()> {
    let mut total_dirs: usize = 0;
    let mut total_files: usize = 0;
    for path in &config.paths {
        println!("{}{}", config.path_size(path), path);
        let (dirs, files) = visit_dir(&config, path, 1, true, "".to_string());
        total_dirs += dirs;
        total_files += files;
    }
    println!(
        "\n{} directories{}",
        total_dirs,
        if config.dir_only {
            "".to_string()
        } else {
            format!(", {} files", total_files)
        }
    );
    Ok(())
}

fn visit_dir(
    config: &Config,
    path: &str,
    depth: usize,
    ancestor_end: bool,
    ancestor_bar: String,
) -> (usize, usize) {
    let mut res: (usize, usize) = (1, 0);
    let mut entries = WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !config.dir_only || e.file_type().is_dir() && config.dir_only)
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
        let size = config.entry_size(&entry);
        if entry.file_type().is_dir() {
            let next_bar = format!(
                "{}{:<s$}",
                ancestor_bar,
                if is_end { "" } else { "│" },
                s = 4
            );
            println!("{}{} {}{}", ancestor_bar, sym, size, name);
            if config.depth.is_none() || config.depth.is_some_and(|d| d > depth) {
                let (next_dir, next_file) = visit_dir(
                    config,
                    entry.path().to_str().unwrap_or_default(),
                    depth + 1,
                    ancestor_end && is_end,
                    next_bar,
                );
                res = (res.0 + next_dir, res.1 + next_file);
            } else {
                res.0 += 1;
            }
        } else if entry.file_type().is_file() {
            println!("{}{} {}{}", ancestor_bar, sym, size, name);
            res.1 += 1;
        } else if entry.file_type().is_symlink() {
            println!(
                "{}{} {}{}",
                ancestor_bar,
                sym,
                size,
                format!(
                    "{} -> {}",
                    name,
                    fs::read_link(entry.path())
                        .unwrap_or_default()
                        .to_str()
                        .unwrap()
                )
            );
            res.1 += 1;
        }
    }
    res
}
