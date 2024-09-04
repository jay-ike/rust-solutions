use clap::{value_parser, Arg, ArgAction, Command};
use std::error::Error;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader, Read};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: isize,
    bytes: Option<isize>,
}

pub fn parse_positive_int(value: &str) -> MyResult<isize> {
    match value.parse::<isize>() {
        Ok(val) if val != 0 => Ok(val),
        _ => Err(value.into()),
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
                .help("input files"),
        )
        .arg(
            Arg::new("bytes")
                .value_name("BYTES")
                .short('c')
                .long("bytes")
                .conflicts_with("lines")
                .action(ArgAction::Set)
                .allow_hyphen_values(true)
                .value_parser(|val: &str| match parse_positive_int(val) {
                    Ok(v) => Ok(v),
                    _ => Err("invalid digit found in string"),
                })
                .help("number of bytes to print"),
        )
        .arg(
            Arg::new("lines")
                .value_name("LINES")
                .short('n')
                .long("lines")
                .default_value("10")
                .value_parser(|val: &str| match parse_positive_int(val) {
                    Ok(v) => Ok(v),
                    _ => Err("invalid digit found in string"),
                })
                .num_args(1)
                .allow_hyphen_values(true)
                .action(ArgAction::Set)
                .help("number of lines to print"),
        )
        .get_matches();
    Ok(Config {
        bytes: matches.get_one::<isize>("bytes").copied(),
        files: matches
            .get_many::<String>("files")
            .expect("at least one file should be specified")
            .map(|v| v.as_str().to_string())
            .into_iter()
            .collect(),
        lines: *matches
            .get_one::<isize>("lines")
            .expect("lines count unavailable"),
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn handle_lines(mut file: Box<dyn BufRead>, arg_lines: isize) -> MyResult<()> {
    let mut buffer = String::new();
    let mut lines: usize = 0;
    let requested_lines = arg_lines.unsigned_abs();
    loop {
        let bytes = file.read_line(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        if arg_lines > 0 {
            print!("{}", buffer);
            buffer.clear();
        }
        lines += 1;
        if lines == arg_lines.unsigned_abs() && arg_lines > 0 {
            break;
        }
    }

    if !buffer.is_empty() && lines > requested_lines {
        buffer.split('\n').enumerate().for_each(
            |(i, content)| {
                if i < lines - requested_lines {
                    print!("{}\n", content);
                }
            }
        );
    }
    Ok(())
}

fn handle_bytes(mut file: Box<dyn BufRead>, arg_bytes: isize) -> MyResult<()> {
    let mut buffer: Vec<u8> = vec![];
    let mut read_bytes: usize = 0;
    loop {
        let mut tmp: Vec<u8>;
        let bytes: usize;
        if arg_bytes > 0 {
            tmp = vec![0; arg_bytes.unsigned_abs().try_into().unwrap()];
            let mut handle = file.take(arg_bytes.unsigned_abs() as u64);
            bytes = handle.read(&mut tmp)?;
            read_bytes += bytes;
            tmp[..bytes].into_iter().for_each(|&bit| buffer.push(bit));
            break;
        } else {
            let abs_bytes = arg_bytes.unsigned_abs();
            tmp = vec![0; 128];
            bytes = file.read(&mut tmp)?;
            read_bytes += bytes;
            tmp[..bytes].into_iter().for_each(|&bit| buffer.push(bit));
            if bytes == 0 {
                read_bytes = if read_bytes > abs_bytes {read_bytes - abs_bytes} else {0};
                break;
            }
        }
    }
    if read_bytes > 0 {
        print!("{}", String::from_utf8_lossy(&buffer[..read_bytes]));
    }
    Ok(())
}
pub fn run(config: Config) -> MyResult<()> {
    let files_count = config.files.len();
    for (index, filename) in config.files.into_iter().enumerate() {
        if files_count > 1 {
            println!("{}==> {} <==", if index > 0 { "\n" } else { "" }, filename);
        }
        match open(&filename) {
            Err(err) => eprintln!("Failed to Open {}: {}", filename, err),
            Ok(file) => {
                if let Some(bytes_count) = config.bytes {
                    handle_bytes(file, bytes_count)?;
                } else {
                    handle_lines(file, config.lines)?;
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
