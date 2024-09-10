use clap::{value_parser, Arg, ArgAction, Command};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

impl Config {
    fn print(
        &self,
        writer: &mut Box<dyn io::Write>,
        content: &mut String,
        dup_count: usize,
    ) -> MyResult<()> {
        if dup_count <= 0 {
            return Ok(());
        }
        if self.count {
            write!(writer, "{:>4} {}", dup_count, *content)?;
        } else {
            write!(writer, "{}", *content)?;
        }
        Ok(())
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn get_args() -> MyResult<Config> {
    let matches = Command::new("uniqr")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .version("0.1.0")
        .about("Rust implementation of uniq")
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .action(ArgAction::SetTrue)
                .help("prefix line by the number of occurences"),
        )
        .arg(
            Arg::new("in_file")
                .value_name("INPUT")
                .help("Input file")
                .value_parser(value_parser!(String))
                .default_value("-")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("out_file")
                .value_name("OUTPUT")
                .help("Output file")
                .value_parser(value_parser!(String))
                .action(ArgAction::Set),
        )
        .get_matches();
    Ok(Config {
        in_file: matches
            .get_one::<String>("in_file")
            .expect("invalid file")
            .to_string(),
        out_file: matches.get_one::<String>("out_file").cloned(),
        count: matches.get_flag("count"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut out_file: Box<dyn io::Write> = match &config.out_file {
        Some(file) => Box::new(File::create(file)?),
        _ => Box::new(io::stdout()),
    };
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut dup_count = 0;
    let mut prev_line = String::new();
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            config.print(&mut out_file, &mut prev_line, dup_count);
            break;
        }
        if prev_line.trim_end() != line.trim_end() {
            config.print(&mut out_file, &mut prev_line, dup_count);
            prev_line = line.clone();
            dup_count = 0;
        }
        dup_count += 1;
        line.clear();
    }
    Ok(())
}
