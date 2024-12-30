use chrono::{Datelike, Local, NaiveDate};
use clap::{value_parser, Arg, ArgAction, Command};
use std::error::Error;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

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
                .value_parser(value_parser!(u32))
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("current-year")
                .short('y')
                .long("year")
                .help("show the whole current year")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("year")
                .value_name("YEAR")
                .help("year to be printed (1-9999)")
                .value_parser(value_parser!(i32))
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
            month: matches.get_one("month").copied().or_else(|| Some(today.month())),
            year: matches.get_one("year").copied().or_else(|| Some(today.year())).unwrap(),
            today: today.date_naive()
        })
    }
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}
