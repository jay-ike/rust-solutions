use std::fs::File;
use std::error::Error;
use std::io::{stdin, BufRead, BufReader, Read};
use clap::{value_parser, Arg, ArgAction, Command};


pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn parse_positive_int(value: &str) -> MyResult<usize>{
    match value.parse::<usize>() {
        Ok(val) if val > 0 => Ok(val),
        _ => Err(value.into())
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("headr")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .version("0.1.0")
        .about("Rust implementation of the GNU head command")
        .arg(
            Arg::new("files")
            .num_args(1..)
            .value_parser(value_parser!(String))
            .default_value("-")
            .value_name("FILES")
            .help("input files")
        )
        .arg(
            Arg::new("bytes")
            .value_name("BYTES")
            .short('c')
            .long("bytes")
            .conflicts_with("lines")
            .action(ArgAction::Set)
            .value_parser(|val: &str| match parse_positive_int(val) {
                Ok(v) => Ok(v),
                _ => Err("invalid digit found in string")
            })
            .help("number of bytes to print")
        )
        .arg(
            Arg::new("lines")
            .value_name("LINES")
            .short('n')
            .long("lines")
            .default_value("10")
            .value_parser(|val: &str| match parse_positive_int(val) {
                Ok(v) => Ok(v),
                _ => Err("invalid digit found in string")
            })
            .action(ArgAction::Set)
            .help("number of lines to print")
        ).get_matches();
    Ok(
        Config {
            bytes: matches.get_one::<usize>("bytes").copied(),
            files: matches.get_many::<String>("files")
                .expect("at least one file should be specified")
                .map(|v| v.as_str().to_string())
                .into_iter()
                .collect(),
            lines: *matches.get_one::<usize>("lines")
                .expect("lines count unavailable")
        }
    )
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let files_count = config.files.len();
    for (index, filename) in config.files.into_iter().enumerate() {
        if files_count > 1 {
            println!(
                "{}==> {} <==",
                if index > 0 {"\n"} else {""},
                filename
            );
        }
        match open(&filename) {
            Err(err) => eprintln!("Failed to Open {}: {}", filename, err),
            Ok(mut file) => {
                if let Some(bytes_count) = config.bytes {
                    let mut handle = file.take(bytes_count as u64);
                    let mut buffer = vec![0; bytes_count];
                    let bytes = handle.read(&mut buffer)?;
                    print!(
                        "{}",
                        String::from_utf8_lossy(&buffer[..bytes])
                    );
                } else {
                    let mut line = String::new();
                    for _ in 0..config.lines {
                        let bytes = file.read_line(&mut line)?;
                        if bytes == 0 {
                            break;
                        }
                        print!("{}", line);
                        line.clear();
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_positive_int;

#[test]
    fn test_parse_positive_int() {
        let mut res;
        res = parse_positive_int("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 3);
        res = parse_positive_int("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo".to_string())
    }
}
