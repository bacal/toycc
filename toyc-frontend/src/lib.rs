use std::io::{BufReader, Read, Seek};
use toycc_api::TccFrontEnd;

pub mod scanner;

pub struct ToyCFrontend<T>
where T: Read + Sync + Seek
{
    reader: Option<BufReader<T>>,
}


impl<T> ToyCFrontend<T>
where T: Read + Sync + Seek
{
    pub fn new() -> Self{
        Self{
            reader: None,
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