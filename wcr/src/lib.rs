use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead, stdin};
use clap::{value_parser, Arg, ArgAction, Command};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    settings: FileParams
}

#[derive(Debug)]
struct  FileParams {
    bytes: bool,
    chars: bool,
    lines: bool,
    words: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_bytes: usize,
    num_chars: usize,
    num_lines: usize,
    num_words: usize
}

impl std::ops::AddAssign for FileInfo {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            num_bytes: self.num_bytes + rhs.num_bytes,
            num_chars: self.num_chars + rhs.num_chars,
            num_lines: self.num_lines + rhs.num_lines,
            num_words: self.num_words + rhs.num_words
        }
    }
}

impl Copy for FileInfo {}

impl Clone for FileInfo {
    fn clone(&self) -> Self {
        *self
    }
}

impl FileParams {
    fn arg_count(&self) -> usize {
        [self.bytes, self.chars, self.lines, self.words]
            .iter().filter(|v| **v).count()

    }
}
impl  FileInfo {
    fn void () -> Self {
        FileInfo {
            num_bytes: 0,
            num_chars: 0,
            num_lines: 0,
            num_words: 0
        }
    }
    fn is_void(&self) -> bool {
        [self.num_bytes, self.num_chars, self.num_lines, self.num_words]
            .iter().all(|&p| p == 0)
    }
    fn max_digits(&self, params: &FileParams) -> usize {
        let args = params.arg_count();
        [
            (params.bytes, self.num_bytes),
            (params.chars, self.num_chars),
            (params.lines, self.num_lines),
            (params.words, self.num_words)
        ]
            .iter()
            .filter(|(arg1, _)| *arg1 || args > 1)
            .fold(0usize, |acc, (_, v2)| acc.max(*v2)).to_string().len()
    }
    fn print (&self, config: &FileParams, width: usize) {
        let mut prev_matched = false;
        if config.lines {
            print!("{:>width$}", self.num_lines);
            prev_matched = true;
        }
        if config.words {
            print!("{1}{:>width$}", self.num_words, if prev_matched {" "} else {""});
            prev_matched = true;
        }
        if config.chars {
            print!("{1}{:>width$}", self.num_chars, if prev_matched  {" "} else {""});
            prev_matched = true;
        }
        if config.bytes {
            print!("{1}{:>width$}", self.num_bytes, if prev_matched {" "} else {""});
        }
    }
}
pub fn count(mut file: impl BufRead) -> MyResult<FileInfo>{
    let mut infos = FileInfo {
        num_bytes: 0,
        num_chars: 0,
        num_lines: 0,
        num_words: 0
    };
    loop {
        let mut buffer = String::new();
        let bytes = file.read_line(&mut buffer)?;

        if bytes == 0 {
            break;
        }
        infos.num_bytes += bytes;
        infos.num_chars += buffer.chars().count();
        infos.num_lines += 1;
        infos.num_words += buffer.split(char::is_whitespace).map(|s| s.trim())
            .filter(|&s| !s.is_empty()).count()
    }
    Ok(infos)
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("wcr")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("Rust version of the wc command")
        .arg(
            Arg::new("files")
            .id("files")
            .num_args(1..)
            .default_value("-")
            .value_name("FILES")
            .value_parser(value_parser!(String))
        )
        .arg(
            Arg::new("words")
            .id("words")
            .short('w')
            .long("words")
            .help("print the word counts")
            .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("lines")
            .id("lines")
            .short('l')
            .long("lines")
            .help("print the line counts")
            .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("bytes")
            .id("bytes")
            .short('c')
            .long("bytes")
            .help("print the byte counts")
            .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("chars")
            .id("chars")
            .short('m')
            .long("chars")
            .help("print the character counts")
            .action(ArgAction::SetTrue)
        ).get_matches();
    let mut lines = matches.get_flag("lines");
    let mut words = matches.get_flag("words");
    let chars = matches.get_flag("chars");
    let mut bytes = matches.get_flag("bytes");

    if [lines, words, chars, bytes].iter().all(|&v| !v) {
        lines = true;
        words = true;
        bytes = true;
    }
    Ok(Config {
        files: matches.get_many::<String>("files")
            .expect("provide at least one file")
            .into_iter().map(|s| s.as_str().to_string())
            .collect(),
        settings: FileParams {
            bytes,
            chars,
            lines,
            words
        }
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>>{
    match filename {
       "-" =>  Ok(Box::new(BufReader::new(stdin()))),
       _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    }

}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_infos = FileInfo::void();
    let mut all_infos: Vec<(FileInfo, &str)> = [].to_vec();
    let mut max_digits: usize = 0;
    for filename in &config.files {
        match open(filename.as_str()) {
            Err(e) => eprintln!("failed to open {}: {}", filename, e),
            Ok(file) => {
                let infos = count(file)?;
                all_infos.push((infos, filename));
                max_digits = max_digits.max(infos.max_digits(&config.settings));
                if filename != "-" {
                    total_infos += infos;
                }
            }
        }
    }
    if *&config.files.len() > 1 {
        max_digits = max_digits.max(3);
    }
    max_digits = max_digits.max(total_infos.max_digits(&config.settings));
    for (infos, filename) in all_infos {
        infos.print(
            &config.settings,
            max_digits.max(config.files.len())
        );
        if filename == "-" {
            print!("\n");
        } else {
            print!(" {}\n", filename);
        }
    }
    if !total_infos.is_void()  && config.files.len() > 1 {
        total_infos.print(&config.settings, max_digits.max(config.files.len()));
        print!(" total\n");
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::FileInfo;

    use super::count;
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        let expected;
        assert!(info.is_ok());
        expected = FileInfo {
            num_bytes: 48,
            num_chars: 48,
            num_lines: 1,
            num_words: 10
        };
        assert_eq!(info.unwrap(), expected);
    }
}
