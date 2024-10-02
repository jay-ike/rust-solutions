use commr::{get_args, run};

fn main() {
    if let Err(e) = get_args().and_then(run)  {
        eprint!("{}", e);
        std::process::exit(1);
    }
}
