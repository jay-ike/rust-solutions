use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate};
use clap::{value_parser, Arg, ArgAction, Command};
use itertools::{izip, Itertools};
use std::{error::Error, str::FromStr, usize};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

const LINE_WIDTH: usize = 22;
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
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
                .value_name("MONTH")
                .conflicts_with("current-year")
                .help("month name or number (1-12)")
                .value_parser(|v: &str| -> Result<u32, String> { parse_month(v) })
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
                .conflicts_with("current-year")
                .value_parser(|v: &str| -> Result<i32, String> {
                    match parse_year(v) {
                        Ok(res) => Ok(res),
                        Err(e) => Err(format!("{}", e)),
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
                .unwrap_or_else(|| today.year()),
            today: today.date_naive(),
        })
    }
}

pub fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

pub fn parse_year(year: &str) -> MyResult<i32> {
    match parse_int::<i32>(year) {
        Ok(res) => {
            if (1..=9999).contains(&res) {
                Ok(res)
            } else {
                Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
            }
        }
        _ => Err(format!("invalid digit found in string").into()),
    }
}

pub fn parse_month(month: &str) -> Result<u32, String> {
    let res = parse_int::<u32>(month).or_else(|_| {
        let res = MONTH_NAMES
            .into_iter()
            .enumerate()
            .filter(|(_, v)| v.to_lowercase().starts_with(&month.to_lowercase()))
            .exactly_one();
        match res {
            Err(_) => Err(format!("Invalid month \"{}\"", &month)),
            Ok((index, _)) => Ok((index + 1).try_into().unwrap()),
        }
    })?;
    if (1..=12).contains(&res) {
        Ok(res)
    } else {
        Err(format!("month \"{}\" not in the range 1 through 12", month).into())
    }
}

pub fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut days: Vec<String> = (1..first.weekday().number_from_sunday())
        .into_iter()
        .map(|_| "  ".to_string())
        .collect();
    let last = last_day_in_month(year, month);
    let is_today =
        |day: u32| -> bool { year == today.year() && month == today.month() && day == today.day() };
    days.extend((first.day()..=last.day()).into_iter().map(|num| {
        let fmt = format!("{:>2}", num);
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));
    let month_name = MONTH_NAMES[month as usize - 1];
    let mut lines = Vec::with_capacity(8);
    lines.push(format!(
        "{:^20}  ",
        if print_year {
            format!("{} {}", month_name, year)
        } else {
            format!("{}", month_name)
        },
    ));
    lines.push("Su Mo Tu We Th Fr Sa  ".to_string());
    for week in days.chunks(7) {
        lines.push(format!(
            "{:width$}  ",
            week.join(" "),
            width = LINE_WIDTH - 2
        ));
    }
    while lines.len() < 8 {
       lines.push(" ".repeat(LINE_WIDTH));
    }
    lines
}

pub fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
}

pub fn run(config: Config) -> MyResult<()> {
    match config.month {
        Some(month) => {
            let lines = format_month(config.year, month, true, config.today);
            println!("{}", lines.join("\n"));
        }
        None => {
            println!("{:>32}", config.year);
            let months: Vec<_> = (1..=12)
                .into_iter()
                .map(|month| {
                    format_month(config.year, month, false, config.today)
                })
                .collect();
            for (i, chunk) in months.chunks(3).enumerate() {
                if let [m1, m2, m3] = chunk {
                    for line in izip!(m1, m2, m3) {
                        println!("{}{}{}", line.0, line.1, line.2);
                    }
                    if i < 3 {
                        println!();
                    }
                }
            }

        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, parse_int, parse_month, parse_year};
    use chrono::NaiveDate;

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

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }
}
