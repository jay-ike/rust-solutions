use std::error::Error;
use clap::{value_parser, Arg, ArgAction, Command};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool
}
struct ParsedFile {
    buffer: Box<dyn BufRead>,
    allow_blank: bool,
    show_lines: bool
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(buffer) => read_lines(ParsedFile {
                buffer,
                allow_blank: config.number_nonblank_lines,
                show_lines: config.number_lines
            })
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let matches =  Command::new("catr")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("Rust implementation of the cat command")
        .arg(
            Arg::new("files")
            .value_name("FILES")
            .num_args(1..)
            .help("input files")
            .default_value("-")
            .value_parser(value_parser!(String))
        )
        .arg(
            Arg::new("count_blanks")
            .short('b')
            .long("number-nonblank")
            .action(ArgAction::SetFalse)
            .help("Don't Print non-blank lines number")
        )
        .arg(
            Arg::new("show_line_number")
            .short('n')
            .long("number")
            .action(ArgAction::SetTrue)
            .conflicts_with("count_blanks")
            .help("Print line numbers")
        ).get_matches();
    Ok(Config {
        files: matches.get_many::<String>("files")
            .expect("files should be specified")
            .into_iter()
            .map(|s| s.as_str().to_string())
            .collect(),
        number_lines: matches.get_flag("show_line_number"),
        number_nonblank_lines: matches.get_flag("count_blanks")
    })
}

pub fn open(filename: &str) -> MyResult<Box<dyn BufRead>>{
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    }
}
fn read_lines(file: ParsedFile) {
    let mut i = 1;
    for line in file.buffer.lines().map(|x| x.unwrap()) {
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
