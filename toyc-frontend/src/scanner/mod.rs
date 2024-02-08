use std::io::{Seek};
use std::iter::Peekable;
use std::str::Chars;

pub mod token;

pub struct ToyCScanner<'a>
{
    line: Peekable<Chars<'a>>,
}