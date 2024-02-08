use std::io::{Read, Seek};

pub mod frontend;
pub mod backend;


pub trait TccFrontEnd {
    type Err;
    fn load_data<T: Read + Sync + Seek>(&mut self, data: T);
}