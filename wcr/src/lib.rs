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

trait Printer {
    fn print (&self, config: &FileParams);
}

trait Voidable {
    fn init () -> Self;
    fn is_void(&self) -> bool;
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

impl Voidable for FileInfo {
    fn init () -> Self {
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
}
impl Printer for FileInfo {
   fn print (&self, config: &FileParams) {
       if config.lines {
           print!("{:>8}", self.num_lines);
       }
       if config.words {
           print!("{:>8}", self.num_words);
       }
       if config.bytes {
           print!("{:>8}", self.num_bytes);
       }
       if config.chars {
           print!("{:>8}", self.num_chars);
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
            .conflicts_with("bytes")

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
    let mut total_infos = FileInfo::init();
    for filename in &config.files {
        match open(filename.as_str()) {
            Err(e) => eprintln!("failed to open {}: {}", filename, e),
            Ok(file) => {
                let infos = count(file)?;
                infos.print(&config.settings);
                if filename != "-" {
                    print!(" {}\n", filename);
                    total_infos += infos;
                } else {
                    print!("\n");
                }
            }
        }
    }
    if !total_infos.is_void()  && config.files.len() > 1 {
        total_infos.print(&config.settings);
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
