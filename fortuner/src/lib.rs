use clap::{value_parser, Arg, ArgAction, Command};
use rand::{rngs::StdRng, seq::SliceRandom, thread_rng, SeedableRng};
use regex::{Regex, RegexBuilder};
use std::{
    error::Error,
    ffi::OsStr,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    u64,
};
use walkdir::WalkDir;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Fortune {
    source: String,
    text: String,
}

pub fn parse_u64(val: &str) -> Result<u64, String> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

pub fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let dat = OsStr::new("dat");
    let mut files = vec![];
    for path in paths {
        match fs::metadata(path) {
            Err(e) => return Err(format!("{}:{}", path, e).into()),
            Ok(_) => files.extend(
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file() && e.path().extension() != Some(dat))
                    .map(|e| e.path().into()),
            ),
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut fortunes = vec![];
    let mut buffer = vec![];
    for path in paths {
        let reader = BufReader::new(fs::File::open(path)?);
        let basename = path.file_name().unwrap().to_string_lossy().into_owned();
        for line in reader.lines().filter_map(Result::ok) {
            if line == "%" {
                if !buffer.is_empty() {
                    fortunes.push(Fortune {
                        source: basename.clone(),
                        text: buffer.join("\n"),
                    });
                    buffer.clear();
                }
            } else {
                buffer.push(line.to_string());
            }
        }
    }
    Ok(fortunes)
}

pub fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    if let Some(val) = seed {
        let mut rng = StdRng::seed_from_u64(val);
        fortunes.choose(&mut rng).map(|f| f.text.to_string())
    } else {
        let mut rng = thread_rng();
        fortunes.choose(&mut rng).map(|f| f.text.to_string())
    }
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
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    if let Some(pattern) = config.pattern {
        let mut prev_source = None;
        for fortune in fortunes.iter().filter(|f| pattern.is_match(&f.text)) {
            if prev_source.as_ref().map_or(true, |s| s != &fortune.source) {
                eprintln!("{}\n", fortune.source);
                prev_source = Some(fortune.source.clone());
            }
            println!("{}", fortune.text);
        }
        if prev_source == None {
            println!(
                "No Fortune found matching the pattern '{}'",
                pattern.to_string()
            );
        }
    } else {
        println!(
            "{}",
            pick_fortune(&fortunes, config.seed)
                .or_else(|| Some("No Fortune found".to_string()))
                .unwrap()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{find_files, parse_u64, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;

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

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );
        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());
        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());
        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));
        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // One input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());
        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
A: A bad idea (bad-eye deer)."
            );
        }
        // Multiple input files
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }
    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];
        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
