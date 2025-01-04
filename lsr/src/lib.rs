use chrono::{DateTime, Local};
use clap::{Arg, ArgAction, Command};
use std::{error::Error, fs, os::unix::fs::MetadataExt, path::PathBuf};
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

pub enum Owner {
    User,
    Group,
    Other,
}

impl Owner {
    pub fn masks(&self) -> [u32; 3] {
        match self {
            Self::User => [0o400, 0o200, 0o100],
            Self::Group => [0o040, 0o020, 0o010],
            Self::Other => [0o004, 0o002, 0o001],
        }
    }
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
        show_hidden: matches.get_flag("all"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(config.paths.as_slice(), config.show_hidden)?;
    if config.long {
        println!("{}", format_output(&files)?);
    } else {
        for file in files {
            println!("{}", file.display());
        }
    }
    Ok(())
}

pub fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut result: Vec<_> = Vec::new();
    for path in paths {
        match fs::metadata(path) {
            Err(e) => eprintln!("{}: {}", path, e),
            Ok(meta) => {
                if meta.is_dir() {
                    for entry in fs::read_dir(path)? {
                        let entry = entry?;
                        let is_hidden = entry
                            .path()
                            .file_name()
                            .map_or(false, |name| name.to_string_lossy().starts_with("."));
                        if show_hidden || !is_hidden {
                            result.push(entry.path());
                        }
                    }
                } else {
                    result.push(PathBuf::from(path));
                }
            }
        }
    }
    Ok(result)
}

fn mk_triplet(mode: u32, owner: Owner) -> String {
    let [read, write, exec] = owner.masks();
    format!(
        "{}{}{}",
        if mode & read == 0 {"-"} else {"r"},
        if mode & write == 0 {"-"} else {"w"},
        if mode & exec == 0 {"-"} else {"x"},
    )
}

/// Given a file mode in octal format like 0o751,
/// return a string like "rwxr-x--x"
pub fn format_mode(mode: u32) -> String {
    format!(
    "{}{}{}",
    mk_triplet(mode, Owner::User),
    mk_triplet(mode, Owner::Group),
    mk_triplet(mode, Owner::Other),
    )
}

pub fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    let fmt = "{:<}{:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);
    for path in paths {
        let meta = path.metadata()?;
        let uid = meta.uid();
        let gid = meta.gid();
        let user = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or_else(|| uid.to_string());
        let group = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().to_string())
            .unwrap_or_else(|| gid.to_string());
        let modified: DateTime<Local> = DateTime::from(meta.modified()?);
        table.add_row(
            Row::new()
                .with_cell(if meta.is_dir() { "d" } else { "-" })
                .with_cell(format_mode(meta.mode()))
                .with_cell(meta.nlink().to_string())
                .with_cell(user)
                .with_cell(group)
                .with_cell(meta.len())
                .with_cell(modified.format("%b %d %y %H:%M"))
                .with_cell(path.display()),
        );
    }
    Ok(format!("{}", table))
}

#[cfg(test)]
mod test {
    use super::{find_files, format_mode};

    #[test]
    fn test_find_files() {
        // Find all non-hidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // Find all entries in a directory including hidden
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }
}
