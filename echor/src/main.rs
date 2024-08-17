use clap::{value_parser, Arg, ArgAction, Command};
fn main() {
    let matches = Command::new("echor")
        .version("0.1.0")
        .author("Ndimah Tchougoua <ndimah22@protonmail.com>")
        .about("Rust echo")
        .arg(
            Arg::new("text")
            .value_name("TEXT")
            .num_args(1..)
            .help("Input text")
            .value_parser(value_parser!(String))
            .required(true)
        ).arg(
            Arg::new("omit_newline")
            .short('n')
            .action(ArgAction::SetTrue)
            .help("Do not print newline")
        ).get_matches();

    let text: Vec<_> = matches.get_many::<String>("text")
        .expect("text is required")
        .into_iter()
        .map(|s| s.as_str())
        .collect();
    let mut ending = "\n";
    if matches.get_flag("omit_newline") {
        ending = "";
    }
    println!("{}{}", text.join(" "), ending);
}
