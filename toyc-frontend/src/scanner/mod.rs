use std::io::{BufReader, Seek};
use std::path::Path;

pub mod token;

pub struct Scanner<T>
where T: Sized + Sync + Send + Seek,
{
    file_name: String,
    bufreader: BufReader<T>,
    lines_read: usize,
}

impl<T> Scanner<T>{

    pub fn new(path: Path) -> Self{
        let name = path.file_name().expect("error file not found");

        Self{

        }
    }
}