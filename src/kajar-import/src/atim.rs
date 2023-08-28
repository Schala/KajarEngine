// Credits: https://www.chronocompendium.com/Term/Atim.html

use bytes::Buf;
use png::{BitDepth, ColorType, Encoder, EncodingError};

use std::{
    fs::{self, File},
    io::{self, BufWriter, Cursor, Read},
};


/// Altered TIM image import error
#[derive(Debug)]
pub enum ATIMErr {
	FileRead(io::Error),
    FileWrite(EncodingError),
	PathWrite,
}

/// Altered TIM image
#[derive(Debug)]
pub struct AlteredTIMImage {
	clut: Vec<u16>,
	idx: Vec<u8>,
}

impl Image for AlteredTIMImage {
	type ImageError = ATIMErr;

	fn load(path: &str) -> Result<AlteredTIMImage, ATIMErr> {
		let mut c = Cursor::new(fs::read(path).map_err(|e| TIMErr::FileRead(e))?);

		let n = c.get_u32_le() as usize;

		let ptrs = (0..n)
			.iter()
			.map(|_| c.get_u32_le() as usize)
			.collect::<Vec<usize>>();
	}

	fn save_png(&self, path: &str) -> Result<(), ATIMErr> {
		Ok(())
	}
}
