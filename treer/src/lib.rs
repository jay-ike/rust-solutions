use clap::{value_parser, Arg, ArgAction, Command};
use regex::Regex;
use std::{
    error::Error,
    fs::{metadata, read_link, Metadata},
    os::unix::fs::PermissionsExt,
};
use walkdir::{DirEntry, WalkDir};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    depth: Option<usize>,
    dir_only: bool,
    patterns: Vec<Regex>,
    paths: Vec<String>,
    size: Option<String>,
    size_printer: Option<SizePrinter>,
    show_perms: bool,
}

#[derive(Debug, Eq, PartialEq)]
enum SizeUnit {
    K,
    M,
    G,
    T,
    P,
}

enum Owner {
    User,
    Group,
    Other,
}

#[derive(Debug)]
enum SizePrinter {
    Bytes,
    Human,
}

impl Owner {
    fn get_masks(&self) -> &[u32; 3] {
        match &self {
            Owner::User => &[0o400, 0o200, 0o100],
            Owner::Group => &[0o040, 0o020, 0o010],
            Owner::Other => &[0o004, 0o002, 0o001],
        }
    }
}

impl SizePrinter {
    fn get_file_size(&self, size: u64) -> String {
        match self {
            SizePrinter::Bytes => format!("{:>11}", size),
            SizePrinter::Human => {
                if size < 1024 {
                    return format!("{:>4}", size);
                }
                format!("{:>4}", evaluate_size(size as f64, SizeUnit::K))
            }
        }
    }
}

impl SizeUnit {
    fn is_parsable(val: &str) -> bool {
        let re = Regex::new(r"^(?<sign>[-+])?(?<factor>\d+)(?<unit>[KMGTP])?$").unwrap();
        re.is_match(val)
    }
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
    pub fn entry_details(&self, entry: Metadata) -> String {
        let mut result = "".to_string();
        let file_size = match &self.size_printer {
            Some(sp) => sp.get_file_size(entry.len()),
            _ => "".to_string(),
        };
        if self.show_perms {
            result += format!(
                "{}{}",
                if entry.is_dir() {
                    "d"
                } else if entry.is_symlink() {
                    "l"
                } else {
                    "-"
                },
                format_mode(entry.permissions().mode())
            )
            .as_str();
        }
        if file_size.len() > 0 {
            result += format!("{}{}", if result.len() > 0 { " " } else { "" }, file_size).as_str();
        }
        if result.len() > 0 {
            return format!("[{}]  ", result);
        }
        result
    }
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
                .action(ArgAction::Set)
                .default_value("."),
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
                .short('S')
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("human_readable")
                .help("Print file size in human readable format")
                .short('H')
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("file-size")
                .help("file size to show")
                .short('s')
                .long("file-size")
                .allow_hyphen_values(true)
                .action(ArgAction::Set)
                .value_parser(|s: &str| match SizeUnit::is_parsable(s) {
                    true => Ok(s.to_string()),
                    _ => Err(format!(
                        "invalid value '{}' for argument --file-size <file-size>",
                        s
                    )),
                })
                .conflicts_with("dir_only"),
        )
        .arg(
            Arg::new("dir_only")
                .help("show only directories")
                .short('d')
                .long("dir-only")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("print_perms")
                .help("show each entry permission")
                .short('p')
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pattern")
                .help("show each file matching the given pattern")
                .num_args(1..)
                .short('P')
                .long("pattern")
                .value_parser(|s: &str| match Regex::new(s) {
                    Ok(_) => Ok(s.to_string()),
                    Err(_) => Err(format!("error invalid value '{}'", s)),
                })
                .action(ArgAction::Append)
                .conflicts_with("dir_only"),
        )
        .get_matches();
    let size = matches.get_one::<String>("file-size");
    let mut size_printer: Option<SizePrinter> = if matches.get_flag("hint_size") {
        Some(SizePrinter::Bytes)
    } else {
        None
    };
    if matches.get_flag("human_readable") || size.is_some_and(|v| SizeUnit::is_parsable(v)) {
        size_printer = Some(SizePrinter::Human);
    }
    Ok(Config {
        depth: matches.get_one::<usize>("depth").copied(),
        dir_only: matches.get_flag("dir_only"),
        patterns: matches
            .get_many::<String>("pattern")
            .unwrap_or_default()
            .map(|s| match Regex::new(s.as_str()) {
                Ok(re) => re,
                _ => panic!("error: ivalid value '{}'", s),
            })
            .into_iter()
            .collect(),
        paths: matches
            .get_many::<String>("dir")
            .unwrap_or_default()
            .map(|v| v.to_string())
            .collect(),
        size: size.cloned(),
        size_printer,
        show_perms: matches.get_flag("print_perms"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_dirs: usize = 0;
    let mut total_files: usize = 0;
    for path in &config.paths {
        println!("{}{}", config.entry_details(metadata(path)?), path);
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
            format!(
                ", {} file{}",
                total_files,
                if total_files > 1 { "s" } else { "" }
            )
        }
    );
    Ok(())
}

fn format_mode(mode: u32) -> String {
    let perm_reader = |mode: u32, owner: Owner| -> String {
        let [read, write, exec] = owner.get_masks();
        format!(
            "{}{}{}",
            if mode & read == 0 { "-" } else { "r" },
            if mode & write == 0 { "-" } else { "w" },
            if mode & exec == 0 { "-" } else { "x" }
        )
    };
    format!(
        "{}{}{}",
        perm_reader(mode, Owner::User),
        perm_reader(mode, Owner::Group),
        perm_reader(mode, Owner::Other)
    )
}

fn get_size_filter(size: Option<String>, file_size: usize) -> bool {
    let re = Regex::new(r"^(?<sign>[+-])?(?<factor>\d+)(?<unit>[KMGTP])?$").unwrap();
    let caps;
    let val;
    let factor;
    if size.is_none() {
        return true;
    }
    val = size.unwrap();
    if !re.is_match(val.as_str()) {
        return false;
    }
    caps = re.captures(val.as_str()).expect("failed to capture group");
    factor = caps
        .name("factor")
        .expect("no factor in size")
        .as_str()
        .parse::<usize>()
        .unwrap();
    let request_size: usize = factor
        * match caps.name("unit") {
            None => 1,
            Some(name) => match name.as_str() {
                "K" => 1024,
                "M" => 1024 * 1024,
                "G" => 1024 * 1024 * 1024,
                "T" => 1024 * 1024 * 1024 * 1024,
                "P" => 1024 * 1024 * 1024 * 1024 * 1024,
                e => unreachable!("unsupported unit '{}'", e),
            },
        };
    match caps.name("sign") {
        None => file_size.eq(&request_size),
        Some(symbol) => match symbol.as_str() {
            "+" => file_size.gt(&request_size),
            "-" => file_size.lt(&request_size),
            s => unreachable!("unsupported symbol '{}'", s),
        },
    }
}

fn visit_dir(
    config: &Config,
    path: &str,
    depth: usize,
    ancestor_end: bool,
    ancestor_bar: String,
) -> (usize, usize) {
    let mut res: (usize, usize) = (1, 0);
    let pattern_filter = |entry: &DirEntry| -> bool {
        config.patterns.is_empty()
            || entry.file_type().is_dir()
            || config
                .patterns
                .iter()
                .any(|re| re.is_match(entry.file_name().to_str().unwrap()))
    };
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
        .filter(pattern_filter)
        .filter(|e| {
            get_size_filter(
                config.size.clone(),
                match e.metadata() {
                    Ok(data) => data.len().try_into().unwrap(),
                    _ => 0,
                },
            ) || e.file_type().is_dir()
        })
        .peekable();
    while let Some(entry) = entries.next() {
        let name = entry.file_name().to_str().unwrap();
        let is_end = entries.peek().is_none();
        let sym = if is_end { "└──" } else { "├──" };
        let size = config.entry_details(entry.metadata().expect("entry should have metadata"));
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
                    read_link(entry.path())
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

#[cfg(test)]
mod tests {
    use crate::format_mode;
    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }
}
