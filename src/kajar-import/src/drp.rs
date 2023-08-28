// Credit: https://www.chronocompendium.com/Term/Drp.html

use bytemuck::{bytes_of_mut, Zeroable};
use bytemuck_derive::{Pod, Zeroable};

use std::{
	collections::HashMap,
    fs::{self, File},
    io::{self, Read},
	path::PathBuf,
};

use crate::tag;

/// File header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Header {
	sig: u32,
	_4: u32,
	n: u16,
	_a: u16,
}

/// Subfile type
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum SubType {
	DRP = 1,
	GenericMesh,
	TIMInfo,
	TIM,
	MInst,
	Unknown07 = 7,
	Unknown0A = 10,
	MDL,
	Unknown0C,
	Unknown10 = 16,
	BattlefieldMesh = 18,
	LightTIMInfo = 21,
	MSeq,
	Anim = 25,
	Unknown1A,
	LZSS = 37,
}

/// Subfile header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct SubHeader {
	_0: u32,
	name: u32,
	kind: u8,
	size: [u8; 3],
}

/// DRP errors
#[derive(Debug)]
pub enum DRPErr {
	FileRead(io::Error),
	FileWrite(io::Error),
	HeaderRead(io::Error),
	Magic(u32),
	ResRead(io::Error),
	SubHeaderRead(io::Error),
}

/// Extracted subfile
#[derive(Debug)]
pub struct DynRes {
	kind: SubType,
	data: Vec<u8>,
}

impl DynRes {
	/// Dumps a file to the specified path
	pub fn dump(&self, path: &str) -> Result<(), DRPErr> {
		let ext = match self.kind {
			SubType::DRP => ".drp",
			SubType::GenericMesh | SubType::BattlefieldMesh => ".mesh",
			SubType::TIMInfo | SubType::LightTIMInfo => ".timinfo",
			SubType::TIM => ".tim",
			SubType::MInst => ".minst",
			SubType::MDL => ".mdl",
			SubType::MSeq => ".mseq",
			SubType::Anim => ".anim",
			SubType::LZSS => ".lz",
			_ => ".dat",
		};

		let mut out_path = PathBuf::from(path);
		out_path.push(ext);

		fs::write(out_path, &self.data[..]).map_err(|e| DRPErr::FileWrite(e))?;

		Ok(())
	}
}

/// Loads a DRP file, returning a hashmap of subfiles
pub fn load_drp(path: &str) -> Result<HashMap<String, DynRes>, DRPErr> {
	let mut buf = fs::read(path).map_err(|e| DRPErr::FileRead(e))?;

	let mut hdr = Header::zeroed();
	buf.read_exact(bytes_of_mut(&mut hdr))
		.map_err(|e| DRPErr::HeaderRead(e))?;

	if hdr.sig != tag!(b"drp\0") {
		return Err(DRPErr::Magic(hdr.sig));
	}

	let n = (hdr.n >> 6) as usize;
	let ptrs = (0..n)
		.iter()
		.map(|_| buf.get_u32_le() as usize)
        .collect::<Vec<usize>>();

	let mut filemap = HashMap::new();
	for _ in 0..n {
		let mut fh = SubHeader::zeroed();
		buf.read_exact(bytes_of_mut(&mut fh))
			.map_err(|e| DRPErr::SubHeaderRead(e))?;

		let kind = match fh.kind {
			1 => SubType::DRP,
			2 => SubType::GenericMesh,
			3 => SubType::TIMInfo,
			4 => SubType::TIM,
			5 => SubType::MInst,
			7 => SubType::Unknown07,
			10 => SubType::Unknown0A,
			11 => SubType::MDL,
			12 => SubType::Unknown0C,
			16 => SubType::Unknown10,
			18 => SubType::BattlefieldMesh,
			21 => SubType::LightTIMInfo,
			22 => SubType::MSeq,
			25 => SubType::Anim,
			26 => SubType::Unknown1A,
			37 => SubType::LZSS,
			_ => unreachable!(),
		};

		let name = fh.name.to_be_bytes();
		let name: String = name.iter().map(|c| *c).collect();

		let size = (u32::from_le_bytes([fh.size[0], fh.size[1], fh.size[2], 0]) as usize) >> 4;
		let mut data = vec![0; size];
		buf.read_exact(&mut data[..]).map_err(|e| DRPErr::ResRead(e))?;

		filemap.insert(name, DynRes { kind, data });
	}

	Ok(filemap)
}
