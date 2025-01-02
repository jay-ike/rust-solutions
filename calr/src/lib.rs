use chrono::{Datelike, Local, NaiveDate};
use clap::{value_parser, Arg, ArgAction, Command};
use itertools::Itertools;
use std::{error::Error, str::FromStr};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "Febuary",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

pub fn get_args() -> MyResult<Config> {
    let today = Local::now();
    let matches = Command::new("calr")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("A rust implementation of the cal command")
        .arg(
            Arg::new("month")
                .short('m')
                .value_name("Month")
                .long("month")
                .help("month name or number (1-12)")
                .value_parser(|v: &str| -> Result<u32, String> {
                    parse_month(v)
                })
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("current-year")
                .short('y')
                .long("year")
                .value_name("SHOW_YEAR")
                .help("show the whole current year")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("year")
                .value_name("YEAR")
                .help("year to be printed (1-9999)")
                .value_parser(value_parser!(i32))
                .value_parser(|v: &str| -> Result<i32, String> {
                    match parse_year(v) {
                        Ok(res) => Ok(res),
                        Err(e) => Err(format!("{}", e))
                    }
                })
                .action(ArgAction::Set),
        )
        .get_matches();
    if matches.get_flag("current-year") {
        Ok(Config {
            month: None,
            year: today.year(),
            today: today.date_naive(),
        })
    } else {
        Ok(Config {
            month: matches
                .get_one("month")
                .copied()
                .or_else(|| Some(today.month())),
            year: matches
                .get_one("year")
                .copied()
                .or_else(|| Some(today.year()))
                .unwrap(),
            today: today.date_naive(),
        })
    }
}

pub fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

pub fn parse_year(year: &str) -> MyResult<i32> {
    let res = parse_int::<i32>(year)?;
    if (1..=9999).contains(&res) {
        Ok(res)
    } else {
        Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
    }
}

pub fn parse_month(month: &str) -> Result<u32, String> {
    let res = parse_int::<u32>(month).or_else(|_| {
        let  res = MONTH_NAMES
            .into_iter()
            .enumerate()
            .filter(|(_, v)| v.to_lowercase().starts_with(&month.to_lowercase()))
            .exactly_one();
        match res {
           Err(_)  => Err(format!("Invalid month \"{}\"", &month)),
            Ok((index, _)) => Ok((index + 1).try_into().unwrap())
        }
    })?;
    if (1..=12).contains(&res) {
        Ok(res)
    } else {
        Err(
            format!("month \"{}\" not in the range 1 through 12", month).into(),
        )
    }
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_int, parse_month, parse_year};
    #[test]
    fn test_parse_int() {
        // Parse positive int as usize
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);
        // Parse negative int as i32
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);
        // Fail on a string
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);
        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);
        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );
        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );
        let res = parse_year("foo");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);
        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );
        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );
        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }
}
