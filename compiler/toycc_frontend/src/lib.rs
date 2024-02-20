mod parser;
mod scanner;
pub use parser::Parser;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

pub struct BufferedStream<S: Read + Seek> {
    reader: BufReader<S>,
    buffer: String,
    eof: bool,
}

impl<S: Read + Seek> BufferedStream<S>
where
    S: Read + Seek,
{
    pub fn new(stream: S) -> Self {
        let reader = BufReader::new(stream);
        Self {
            reader,
            buffer: String::new(),
            eof: false,
        }
    }
    pub fn peek(&mut self) -> Option<String> {
        match self.eof {
            true => None,
            false => Some(self.buffer.to_string()),
        }
    }
}

impl<S: Read + Seek> Iterator for BufferedStream<S> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        match self.reader.read_line(&mut self.buffer) {
            Ok(num) => match num {
                0 => {
                    self.eof = true;
                    None
                }
                _ => Some(self.buffer.to_string()),
            },
            Err(_) => None,
        }
    }
}

impl<S: Read + Seek> Seek for BufferedStream<S> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }

    fn rewind(&mut self) -> std::io::Result<()> {
        self.reader.rewind()
    }
}

impl<S: Read + Seek> Read for BufferedStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}
