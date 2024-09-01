use std::error::Error;

pub trait FileRunService {
    fn run_file(&self, file: &str) -> Result<(), Box<dyn Error>>;
}