use std::error::Error;


pub type MyResult<T> = Result<T, Box<dyn Error>>;

pub fn parse_positive_int(value: &str) -> MyResult<usize>{
    match value.parse::<usize>() {
        Ok(val) if val > 0 => Ok(val),
        _ => Err(value.into())
    }
}
