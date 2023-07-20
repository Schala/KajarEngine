use nom::{
  IResult,
  error::ParseError,
  combinator::value,
  sequence::delimited,
  character::complete::multispace0,
};

use std::io::{
	self,
	Read
};

/// Converts a 4-byte string into a 32-bit big endian integer.
/// Byte strings longer than 4 bytes are truncated.
#[macro_export]
macro_rules! tag {
	($b4: literal) => {
		u32::from_be_bytes([$b4[3], $b4[2], $b4[1], $b4[0]])
	}
}

/// Reads a null-terminated string from a buffer
pub fn read_cstr(mut buf: impl Read) -> io::Result<String> {
	let mut s = String::new();
	let mut b = [0; 1];

	while b[0] != 0 {
		buf.read_exact(&mut b)?;
		if b[0] != 0 {
			s.push(b[0] as char);
		}
	}

	Ok(s)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
	F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
	delimited(multispace0, inner,multispace0)
}
