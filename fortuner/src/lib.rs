use clap::{value_parser, Arg, ArgAction, Command};
use regex::{Regex, RegexBuilder};
use std::{error::Error, u64};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn parse_u64(val: &str) -> Result<u64, String> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("fortuner")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("Rust fortune")
        .arg(
            Arg::new("sources")
                .num_args(1..)
                .action(ArgAction::Set)
                .value_parser(value_parser!(String))
                .required(true),
        )
        .arg(
            Arg::new("seed")
                .short('s')
                .long("seed")
                .action(ArgAction::Set)
                .help("Random seed")
                .value_name("SEED")
                .value_parser(parse_u64),
        )
        .arg(
            Arg::new("pattern")
                .short('m')
                .long("pattern")
                .help("Pattern")
                .value_name("PATTERN")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("case")
                .short('i')
                .long("insensitive")
                .action(ArgAction::SetTrue),
        )
        .get_matches();
    Ok(Config {
        sources: matches
            .get_many::<String>("sources")
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect(),
        pattern: match matches.get_one::<String>("pattern") {
            None => None,
            Some(val) => Some(
                RegexBuilder::new(&val)
                    .case_insensitive(matches.get_flag("case"))
                    .build()
                    .map_err(|_| format!("Invalid --pattern \"{}\"", val))?,
            ),
        },
        seed: matches.get_one::<u64>("seed").copied(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::parse_u64;
    #[test]
    fn test_parse_u64() {
        let res = parse_u64("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "\"a\" not a valid integer");
        let res = parse_u64("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);
        let res = parse_u64("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }
}
