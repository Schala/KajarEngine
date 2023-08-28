use bytes::Buf;

use nom::{
    character::complete::multispace0, combinator::value, error::ParseError, sequence::delimited,
    IResult,
};

use std::{
    fs,
    io::{self, Read},
};

pub mod cc;
pub mod ct;

#[cfg(feature = "ct_win")]
pub mod
/// Converts a 4-byte string into a 32-bit big endian integer.
/// Byte strings longer than 4 bytes are truncated.
#[macro_export]
macro_rules! tag {
    ($b4: literal) => {
        u32::from_be_bytes([$b4[3], $b4[2], $b4[1], $b4[0]])
    };
}

/// Image import/export functionality
pub trait Image {
    type ImageError;

    /// Loads in an image file
    fn load(path: &str) -> Result<Self, ImageError>;

    /// Saves the imported image to a PNG file
    fn save_png(&self, path: &str) -> Result<(), ImageError> {
}

/// Reads a null-terminated string from a buffer
pub fn read_cstr(mut buf: impl Read) -> io::Result<String> {
    let mut s = String::new();
    loop {
        let c = buf.get_u8();
        if c != 0 {
            s.push(c as char);
        } else {
            break;
        }
    }

    Ok(s)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}
