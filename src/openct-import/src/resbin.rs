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
	io::{
		Cursor,
		Read,
		self
	},
	mem::{
		MaybeUninit,
		size_of
	},
	path::PathBuf,
	ptr::addr_of_mut
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
	pub header: Header,
	pub entdata: Vec<ResEntry>,
	pub entries: HashMap<PathBuf, Vec<u8>>,
}

#[derive(Debug)]
pub enum ResBinErr {
	CmpRead(io::Error),
	Decmp(c_int),
	EntryDataRead(PathBuf, io::Error),
	EntryRead(io::Error),
	HeaderMismatch(u32),
	HeaderRead(io::Error),
	Inflate(io::Error),
	PathName(ResEntry),
}

impl ResBin {
	pub fn new(buf: &[u8]) -> Result<ResBin, ResBinErr> {
		let mut header = Header::zeroed();
		let mut cur = Cursor::new(buf);

		if let Err(e) = cur.read_exact(bytes_of_mut(&mut header)) {
			return Err(ResBinErr::HeaderRead(e));
		}

		decode(0, bytes_of_mut(&mut header));

		if header.sig != tag!(b"ARC1") {
			return Err(ResBinErr::HeaderMismatch(header.sig));
		}

		let mut cmp = vec![0; header.cmp_size as usize];
		cur.set_position(header.offs as u64);
		if let Err(e) = cur.read_exact(&mut cmp[..]) {
			return Err(ResBinErr::CmpRead(e));
		}

		decode(header.offs, &mut cmp[..]);
		let dcmp = decompress(&mut cmp[4..], header.size as usize)?;

		let mut c = Cursor::new(&dcmp[..]);
		let n = c.get_u32_le();
		let mut entdata = vec![ResEntry::zeroed(); n as usize];

		for ent in entdata.iter_mut() {
			if let Err(e) = c.read_exact(bytes_of_mut(ent)) {
				return Err(ResBinErr::EntryRead(e));
			}
		}

		let mut entries = HashMap::with_capacity(n as usize);
		for ent in entdata.iter() {
			c.set_position(ent.path_offs as u64);

			if let Ok(s) = read_cstr(&mut c) {
				let path = PathBuf::from(s);
				dbg!(&path);
				let mut data = vec![0; ent.size as usize];

				cur.set_position(ent.data_offs as u64);
				if let Err(e) = cur.read_exact(&mut data[..]) {
					return Err(ResBinErr::EntryDataRead(path, e));
				}
				entries.insert(path, data);
			} else {
				return Err(ResBinErr::PathName(ent.clone()));
			}
		}

		Ok(ResBin { header, entdata, entries })
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

#[cfg(test)]
mod test {
	/*#[test]
	fn test_resbin_extract() {
		use std::fs;

		let buf = fs::read("/Users/admin/Desktop/resources.bin").unwrap();
		let resb = super::ResBin::new(&buf).unwrap();
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
