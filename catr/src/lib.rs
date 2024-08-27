use clap::{value_parser, Arg, ArgAction, Command};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
    squeeze_blank: bool,
}
struct ParsedFile {
    buffer: Box<dyn BufRead>,
    allow_blank: bool,
    show_lines: bool,
    squeeze_blank: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(buffer) => read_lines(ParsedFile {
                buffer,
                allow_blank: config.number_nonblank_lines,
                show_lines: config.number_lines,
                squeeze_blank: config.squeeze_blank,
            }),
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("catr")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("Rust implementation of the cat command")
        .arg(
            Arg::new("files")
                .value_name("FILES")
                .num_args(1..)
                .help("input files")
                .default_value("-")
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("count_blanks")
                .short('b')
                .long("number-nonblank")
                .action(ArgAction::SetFalse)
                .help("number non-empty output lines"),
        )
        .arg(
            Arg::new("squeeze-blank")
                .short('s')
                .long("squeeze-blank")
                .action(ArgAction::SetTrue)
                .help("suppress repeated empty output lines"),
        )
        .arg(
            Arg::new("show_line_number")
                .short('n')
                .long("number")
                .action(ArgAction::SetTrue)
                .conflicts_with("count_blanks")
                .help("number all output lines"),
        )
        .get_matches();
    Ok(Config {
        files: matches
            .get_many::<String>("files")
            .expect("files should be specified")
            .into_iter()
            .map(|s| s.as_str().to_string())
            .collect(),
        number_lines: matches.get_flag("show_line_number"),
        number_nonblank_lines: matches.get_flag("count_blanks"),
        squeeze_blank: matches.get_flag("squeeze-blank"),
    })
}

pub fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
fn read_lines(file: ParsedFile) {
    let mut i = 1;
    let mut prev_blank = 10;
    for (line_num, line) in file.buffer.lines().map(|x| x.unwrap()).enumerate() {
        if prev_blank + 1 == line_num && line.trim().is_empty() && file.squeeze_blank {
            prev_blank = line_num;
            continue;
        }
        if line.trim().is_empty() {
            prev_blank = line_num;
        }
        if !file.show_lines && file.allow_blank {
            println!("{}", line);
            continue;
        }
        if !file.allow_blank && line.trim().is_empty() {
            println!("{}", line);
        } else {
            println!("{:>6}\t{}", i, line);
            i += 1;
        }
    }
}
