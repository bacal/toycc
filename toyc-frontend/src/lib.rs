use std::io::{BufReader, Read, Seek};
use toycc_api::TccFrontEnd;
use crate::scanner::ToyCScanner;

pub mod scanner;

pub struct ToyCFrontend<T>
where T: Read + Sync + Seek
{
    scanner: Option<ToyCScanner<T>>
}


impl<T> ToyCFrontend<T>
where T: Read + Sync + Seek
{
    pub fn new() -> Self{
        Self{
            scanner: None,
        }
    }
}


// impl<T: Read + Sync + Seek> TccFrontEnd for ToyCFrontend<T>{
//     type Err = ();
//
//     fn load_data(&mut self, stream: T){
//         self.reader =  Some(BufReader::new(stream));
//     }
// }