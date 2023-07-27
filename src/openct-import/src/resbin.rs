// Credit to https://github.com/jimzrt/ChronoMod

use bytes::Buf;

use bytemuck::{
	bytes_of_mut,
	Zeroable,
};

use bytemuck_derive::{
	Pod,
	Zeroable,
};

use libz_sys::{
	Bytef,
	inflate,
	inflateEnd,
	inflateInit2_,
	uInt,
	Z_FINISH,
	Z_OK,
	z_stream,
	Z_STREAM_END,
	zlibVersion,
};

use std::{
	collections::HashMap,
	ffi::c_int,
	fs,
	io::{
		Cursor,
		Read,
		self,
	},
	mem::{
		MaybeUninit,
		size_of,
	},
	path::PathBuf,
	ptr::addr_of_mut,
};

use crate::{
	read_cstr,
	tag
};

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Header {
	sig: u32,
	size: u32,
	offs: u32,
	cmp_size: u32,
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct ResEntry {
	path_offs: u32,
	data_offs: u32,
	size: u32,
}

#[derive(Debug)]
pub struct ResBin {
	header: Header,
	entdata: Vec<ResEntry>,
	entries: HashMap<PathBuf, Vec<u8>>,
}

#[derive(Debug)]
pub enum ResBinErr {
	CmpRead(io::Error),
	Decmp(c_int),
	Dump(io::Error),
	EntryDataDecmp(PathBuf, c_int),
	EntryDataRead(PathBuf, io::Error),
	EntryPath(PathBuf),
	EntryRead(io::Error),
	FilePath(PathBuf),
	HeaderMismatch(u32),
	HeaderRead(io::Error),
	Inflate(io::Error),
	PathName(ResEntry),
}

impl ResBin {
	/// Loads all data from resources.bin
	pub fn load(filepath: &str) -> Result<ResBin, ResBinErr> {
		let buf = fs::read(&filepath);
		if buf.is_err() {
			return Err(ResBinErr::FilePath(PathBuf::from(filepath)));
		}

		let mut header = Header::zeroed();
		let mut fc = Cursor::new(buf.unwrap());

		// header
		if let Err(e) = fc.read_exact(bytes_of_mut(&mut header)) {
			return Err(ResBinErr::HeaderRead(e));
		}

		decode(0, bytes_of_mut(&mut header));

		if header.sig != tag!(b"ARC1") {
			return Err(ResBinErr::HeaderMismatch(header.sig));
		}

		// compressed data
		let mut cmp = vec![0; header.cmp_size as usize];
		fc.set_position(header.offs as u64);
		if let Err(e) = fc.read_exact(&mut cmp[..]) {
			return Err(ResBinErr::CmpRead(e));
		}

		decode(header.offs, &mut cmp[..]);
		let dcmp = decompress(&mut cmp[4..], header.size as usize)?;

		// decompressed data
		let mut dc = Cursor::new(&dcmp[..]);
		let n = dc.get_u32_le();
		let mut entdata = vec![ResEntry::zeroed(); n as usize];

		for ent in entdata.iter_mut() {
			if let Err(e) = dc.read_exact(bytes_of_mut(ent)) {
				return Err(ResBinErr::EntryRead(e));
			}
		}

		// entries
		let mut entries = HashMap::with_capacity(n as usize);
		for ent in entdata.iter() {
			dc.set_position(ent.path_offs as u64);

			if let Ok(s) = read_cstr(&mut dc) {
				let path = PathBuf::from(s);
				let mut cdata = vec![0; ent.size as usize];

				fc.set_position(ent.data_offs as u64);
				if let Err(e) = fc.read_exact(&mut cdata[..]) {
					return Err(ResBinErr::EntryDataRead(path, e));
				}

				decode(ent.data_offs, &mut cdata);
				let size = get_u32_le(&cdata[..]) as usize;
				let ddata = decompress(&mut cdata[4..], size)?;

				entries.insert(path, ddata);
			} else {
				return Err(ResBinErr::PathName(ent.clone()));
			}
		}

		Ok(ResBin { header, entdata, entries })
	}

	/// Dumps the contents of a single entry to file.
	pub fn dump(&self, in_path: &str, out_path: &str) -> Result<(), ResBinErr> {
		if let Some(ent) = self.entries.get(&PathBuf::from(in_path)) {
			let mut path = PathBuf::from(out_path);
			path.push(in_path);

			if let Err(e) = fs::write(path.as_path(), ent) {
				return Err(ResBinErr::Dump(e));
			} else {
				return Ok(());
			}
		} else {
			return Err(ResBinErr::EntryPath(PathBuf::from(in_path)));
		}
	}

	/// Dumps all files in resources.bin
	pub fn dump_all(&self, out_path: &str) -> Result<(), ResBinErr> {
		for (p, d) in self.entries.iter() {
			if let Some(path) = p.to_str() {
				self.dump(path, out_path)?;
			}
		}

		Ok(())
	}
}

/// Decodes a block of data
fn decode(offs: u32, data: &mut [u8]) {
	// Decoding uses a common PRNG algorithm
	let mut seed = 0x19000000 + offs;
	data.iter_mut().for_each(|b| {
		seed = seed.wrapping_mul(0x41C64E6D).wrapping_add(12345);
		*b = ((*b as u32) ^ seed >> 24) as u8;
	});
}

/// Inflates zlib-compressed data
fn decompress(data: &mut [u8], dcmp_size: usize) -> Result<Vec<u8>, ResBinErr> {
	let mut dcmp = vec![0; dcmp_size];

	unsafe {
		let zs_ = MaybeUninit::<z_stream>::zeroed();
		let mut zs = zs_.assume_init();
		let ver = zlibVersion();

		zs.next_in = data.as_mut_ptr() as *mut Bytef;
		zs.avail_in = data.len() as uInt;
		zs.next_out = dcmp.as_mut_ptr() as *mut Bytef;
		zs.avail_out = dcmp_size as uInt;

		// decompression uses a custom window of 31 bits
		let err = inflateInit2_(addr_of_mut!(zs), 31, ver, size_of::<z_stream>() as c_int);
		if err != Z_OK {
			return Err(ResBinErr::Decmp(err));
		}

		let err = inflate(addr_of_mut!(zs), Z_FINISH);
		if err != Z_STREAM_END {
			return Err(ResBinErr::Decmp(err));
		}

		inflateEnd(addr_of_mut!(zs));
	}

	Ok(dcmp)
}

/// Helper function to get an unsigned 32-bit value from the start of a buffer
fn get_u32_le(buf: &[u8]) -> u32 {
	u32::from_le_bytes([buf[3], buf[2], buf[1], buf[0]])
}

#[cfg(test)]
mod test {
	/*#[test]
	fn test_resbin_extract() {
		use std::fs;

		let buf = fs::read("/Users/admin/Desktop/resources.bin").unwrap();
		let resb = super::ResBin::load(&buf).unwrap();
		assert_eq(resb.is_ok());
	}*/
}

/*/// Decodes a file buffer, given a key buffer
fn decode_file_with_key(key: &[u32], data: &[u8]) -> Result<Vec<u8>, E> {
	let header = [data[0] ^ 0x75,
		data[1] ^ 0xFA,
		data[2] ^ 0x29,
		data[3] ^ 0x95,
		data[4] ^ 0x05,
		data[5] ^ 0x4D,
		data[6] ^ 0x41,
		data[7] ^ 0x5F];
}

/// Decryption depth 1
fn decrypt1(key: &[u32], data: &[u8]) -> Result<Vec<u8>, E> {
	let out_buf = [0; 8];
	let mut buf_idx = 0;
}*/
