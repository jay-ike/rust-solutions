use crate::Extract::*;
use clap::{value_parser, Arg, ArgAction, Command};
use csv::{ReaderBuilder, StringRecord};
use regex::Regex;
use std::num::NonZeroUsize;
use std::{
    error::Error,
    io::{BufRead, BufReader},
    ops::Range,
};

pub type MyResult<T> = Result<T, Box<dyn Error>>;
pub type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

impl Extract {
    fn is_bytes(&self) -> bool {
        match self {
            Bytes(_) => true,
            _ => false,
        }
    }
}
impl Config {
    fn print(&self, reader: &mut Box<dyn BufRead>) {
        let mut buffer = String::new();
        match &self.extract {
            Bytes(ranges) | Chars(ranges) => loop {
                let bytes = reader.read_line(&mut buffer).unwrap();
                if bytes == 0 {
                    break;
                }
                println!(
                    "{}",
                    if self.extract.is_bytes() {
                        extract_bytes(buffer.as_str(), ranges)
                    } else {
                        extract_chars(buffer.as_str(), ranges)
                    }
                );
                buffer.clear();
            },
            Fields(ranges) => {
                let mut delim_reader = ReaderBuilder::new()
                    .has_headers(false)
                    .delimiter(self.delimiter)
                    .from_reader(reader);
                for record in delim_reader.records() {
                    if let Ok(line) = record {
                        println!(
                            "{}",
                            extract_fields(&line, ranges).join(
                                String::from_utf8_lossy(&[self.delimiter])
                                    .to_string()
                                    .as_str(),
                            )
                        );
                    }
                }
            }
        }
    }
}

pub fn get_args() -> MyResult<Config> {
    let extract: Extract;
    let matches = Command::new("cutr")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .version("0.1.0")
        .about("rust implementation of the cut command")
        .arg(
            Arg::new("bytes")
                .help("selected bytes")
                .short('b')
                .long("bytes")
                .value_name("BYTES")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("chars")
                .help("selected characters")
                .short('c')
                .long("chars")
                .conflicts_with("bytes")
                .value_name("CHARS")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("delimiter")
                .help("Field delimiter")
                .short('d')
                .long("delim")
                .value_name("DELIMITER")
                .value_parser(|v: &str| match v.as_bytes().len() == 1 {
                    true => Ok(*v.as_bytes().first().unwrap()),
                    _ => Err(format!("--delim \"{}\" must be a single byte", v)),
                })
                .default_value("\t")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("fields")
                .help("selected fields")
                .short('f')
                .long("fields")
                .conflicts_with_all(["chars", "bytes"])
                .value_name("FIELDS")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("files")
                .help("Input file(s)")
                .num_args(1..)
                .value_name("FILE")
                .default_value("-")
                .value_parser(value_parser!(String))
                .action(ArgAction::Set),
        )
        .get_matches();
    if let Some(val) = matches.get_one::<String>("chars").cloned() {
        extract = Chars(parse_pos(&val)?);
    } else if let Some(val) = matches.get_one::<String>("bytes").cloned() {
        extract = Bytes(parse_pos(&val)?);
    } else if let Some(val) = matches.get_one::<String>("fields").cloned() {
        extract = Fields(parse_pos(&val)?);
    } else {
        unreachable!("query type not implemented");
    }
    Ok(Config {
        files: matches
            .get_many::<String>("files")
            .expect("should provide files")
            .map(|s| s.as_str().to_string())
            .collect(),
        delimiter: matches.get_one::<u8>("delimiter").cloned().unwrap(),
        extract,
    })
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(filename)?))),
    }
}
pub fn run(config: Config) -> MyResult<()> {
    for file in &config.files {
        match open(file.as_str()) {
            Err(e) => eprintln!("{}: {}", file, e),
            Ok(mut reader) => {
                config.print(&mut reader);
            }
        }
    }
    Ok(())
}
fn parse_index(input: &str) -> Result<usize, String> {
    let val_error = || format!("illegal list value: \"{}\"", input);
    match input.parse::<NonZeroUsize>() {
        Ok(n) if !input.starts_with('+') => Ok(usize::from(n) - 1),
        _ => Err(val_error()),
    }
}
fn parse_pos(range: &str) -> MyResult<PositionList> {
    let re = Regex::new(r"^(\d+)-(\d+)$").unwrap();
    range
        .split(',')
        .into_iter()
        .map(|v| {
            parse_index(v).map(|n| n..n + 1).or_else(|e| {
                re.captures(v).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        return Err(format!(
                            "First number in range ({}) \
                                must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1
                        ));
                    }
                    Ok(n1..n2 + 1)
                })
            })
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}
fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars: Vec<_> = line.chars().collect();
    let mut result: Vec<char> = vec![];
    for range in char_pos.iter().cloned() {
        for i in range {
            if let Some(value) = chars.get(i) {
                result.push(*value);
            }
        }
    }
    result.iter().collect()
}
fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes = line.as_bytes();
    let mut result: Vec<u8> = vec![];
    for range in byte_pos.iter().cloned() {
        for i in range {
            if let Some(byte) = bytes.get(i) {
                result.push(*byte);
            }
        }
    }
    String::from_utf8_lossy(&result).to_string()
}
fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for range in field_pos.iter().cloned() {
        for i in range {
            if let Some(field) = record.get(i) {
                result.push(field.to_string());
            }
        }
    }
    result
}

#[cfg(test)]
mod unit_tests {
    use super::{extract_bytes, extract_chars, extract_fields, parse_pos};
    use csv::StringRecord;
    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("").is_err());
        // Zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);
        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);
        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);
        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);
        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);
        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);
        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);
        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);
        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());
        let res = parse_pos(",");
        assert!(res.is_err());
        let res = parse_pos("1,");
        assert!(res.is_err());
        let res = parse_pos("1-");
        assert!(res.is_err());
        let res = parse_pos("1-1-1");
        assert!(res.is_err());
        let res = parse_pos("1-1-a");
        assert!(res.is_err());
        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );
        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );
        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("1,7,3-5");
        println!("{:?}", res);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);
        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }
    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }
    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }
}
