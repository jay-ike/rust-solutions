use std::error::Error;

pub type MyResult<T> = Result<T, Box<dyn Error>>;

pub struct Config {}

pub fn get_args() -> MyResult<Config>{
    unimplemented!()
}
pub fn run(config: Config) -> MyResult<()> {
    unimplemented!()
}
