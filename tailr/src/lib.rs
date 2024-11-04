use crate::TakeValue::*;
use clap::{value_parser, Arg, ArgAction, Command};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

static REGEX: OnceCell<Regex> = OnceCell::new();

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

fn parse_num(val: &str) -> Result<TakeValue, String> {
    let re = REGEX.get_or_init(|| Regex::new(r"^(?<sign>[+-])?(?<value>\d+)$").unwrap());
    match re.captures(val) {
        Some(caps) => {
            let sign = caps.name("sign").map_or("-", |s| s.as_str());
            let num = format!("{}{}", sign, caps.name("value").unwrap().as_str());
            if let Ok(parsed_val) = num.parse() {
                if sign == "+" && parsed_val == 0 {
                    Ok(PlusZero)
                } else {
                    Ok(TakeNum(parsed_val))
                }
            } else {
                Err(val.into())
            }
        }
        _ => Err(val.into()),
    }
}
pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("tailr")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .version("0.1.0")
        .about("A Rust implementation of the tail command")
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Suppress headers")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .value_name("BYTES")
                .help("Number of bytes")
                .allow_hyphen_values(true)
                .action(ArgAction::Set)
                .value_parser(|val: &str| match parse_num(val) {
                    Ok(res) => Ok(res),
                    _ => Err(format!("illegal byte count -- {}", val)),
                })
                .conflicts_with("lines"),
        )
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .value_name("LINES")
                .help("Number of lines")
                .action(ArgAction::Set)
                .allow_hyphen_values(true)
                .value_parser(|val: &str| match parse_num(val) {
                    Ok(res) => Ok(res),
                    _ => Err(format!("illegal line count -- {}", val)),
                })
                .default_value("10"),
        )
        .arg(
            Arg::new("files")
                .value_name("FILES")
                .num_args(1..)
                .action(ArgAction::Set)
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .get_matches();
    Ok(Config {
        files: matches
            .get_many::<String>("files")
            .unwrap_or_default()
            .map(|v| v.to_string())
            .collect(),
        lines: matches
            .get_one::<TakeValue>("lines")
            .expect("lines should be provided")
            .clone(),
        bytes: matches.get_one::<TakeValue>("bytes").cloned(),
        quiet: matches.get_flag("quiet"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let files = config.files.len();
    for (index, filename) in config.files.iter().enumerate() {
        match File::open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                let (lines, bytes) = count_lines_bytes(&filename)?;
                let reader = BufReader::new(file);
                if !config.quiet && files > 1 {
                    println!("{}==> {} <==", if index > 0 { "\n" } else { "" }, filename);
                }
                if config.bytes.is_some() {
                    print_bytes(reader, &config.bytes.clone().unwrap(), bytes)?;
                } else {
                    print_lines(reader, &config.lines, lines)?;
                }
            }
        }
    }
    Ok(())
}

pub fn print_lines(
    mut file: impl BufRead,
    num_lines: &TakeValue,
    total_lines: i64,
) -> MyResult<()> {
    let seek_index = get_start_index(num_lines, total_lines);
    if seek_index.is_some_and(|seek| seek >= 0) {
        let start_line = seek_index.unwrap();
        let mut buf = String::new();
        let mut line = 0;
        loop {
            let bytes_read = file.read_line(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
            if line >= start_line {
                print!("{}", buf);
            }
            line += 1;
            buf.clear();
        }
    }
    Ok(())
}

pub fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> MyResult<()> {
    let seek_index = get_start_index(num_bytes, total_bytes);
    let mut buf = vec![];
    if seek_index.is_some_and(|seek| seek >= 0) {
        file.seek(SeekFrom::Start(seek_index.unwrap() as u64))?;
        loop {
            let bytes_read = file.read_to_end(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
            print!("{}", String::from_utf8_lossy(&buf));
            buf.clear();
        }
    }
    Ok(())
}

pub fn get_start_index(take_val: &TakeValue, total: i64) -> Option<i64> {
    match take_val {
        PlusZero => {
            if total > 0 {
                return Some(0);
            }
            None
        }
        TakeNum(val) => {
            if *val == 0 {
                return None;
            }
            if val.is_positive() {
                if *val <= total {
                    return Some(*val - 1);
                }
                return None;
            }
            if val + total >= 0 {
                return Some(val + total);
            }
            Some(0)
        }
    }
}

pub fn count_lines_bytes(path: &str) -> MyResult<(i64, i64)> {
    let mut file = BufReader::new(File::open(path)?);
    let mut lines = 0;
    let mut bytes = 0;
    let mut buf = Vec::new();
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        lines += 1;
        bytes += bytes_read as i64;
        buf.clear();
    }
    Ok((lines, bytes))
}

#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, get_start_index, parse_num, TakeValue::*};
    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));
        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));
        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));
        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));
        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);
        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));
        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));
        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));
        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");
        // Any noninteger string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok_and(|val| val == (1, 24)));
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);
        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));
        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);
        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);
        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);
        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));
        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));
        // When starting line/byte is negative and more than total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}
