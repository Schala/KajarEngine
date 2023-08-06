// based on https://github.com/vgmstream/vgmstream/blob/master/src/meta/sqex_sead.c

use anyhow::Result;
use bytes::Buf;

use bytemuck::{
	bytes_of_mut,
	Pod,
	Zeroable
};

use std::{
	collections::HashMap,
	io::{
		Cursor,
		Read
	}
};

use crate::tag;

/// SEAD file header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Header {
	id: u32,
	_ver: u8,
	_flags: u8,
	_chunk_size: u16,
	nchunks: u8,
	filename_size: u8,
	_0a: u16,
	file_size: u32,
}

/// SEAD chunk ID
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
enum ChunkType {
	Instruments = tag!(b"inst"),
	Materials = tag!(b"mtrl"),
	Music = tag!(b"musc"),
	Sequences = tag!(b"seq "),
	Sounds = tag!(b"snd "),
	Tracks = tag!(b"trk "),

	#[default]
	Unknown,
}

impl From<u32> for ChunkType {
	fn from(value: u32) -> Self {
		match value {
			tag!(b"inst") => ChunkType::Instruments,
			tag!(b"mtrl") => ChunkType::Materials,
			tag!(b"musc") => ChunkType::Music,
			tag!(b"seq ") => ChunkType::Sequences,
			tag!(b"snd ") => ChunkType::Sounds,
			tag!(b"trk ") => ChunkType::Tracks,
			_ => ChunkType::Unknown,
		}
	}
}

/// SEAD chunk table metadata
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct ChkTblEntry {
	id: u32,
	_ver: u8,
	_05: u8,
	_size: u16,
	offs: u32,
	_0c: u32,
}

/// SEAD sound header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct SndHdr {
	ver: u8,
	_work: u8,
	size: u16,
	_type: u8,
	nseqs: u8,
	_cat: u8,
	_priority: u8,
	_n: u16,
	_start_end_macro: u16,
	_vol: f32,
	_cycle_cfg: [u8; 10],
	seq_start: u16,
	_audible_range: f32,
	_output: u8,
	_curve: u8,
	_port: u8,
	_name_size: u8,
	_play_len: u32,
}

/// SEAD sequence entry
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct SeqInfo {
	_ver: u8,
	_01: u8,
	size: u16,
	idx: u16,
	_id: u16,
	_08: u16,
}

/// SEAD old sequence
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct OldSeq {
	_cfg: [u8; 12],
	_n: u16,
	_volpitch: u16,
	cmd_start: u16,
	_18: u16,
}

/// SEAD new sequence
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct NewSeq {
	_id: u16,
	cmd_start: u16,
	_n: u8,
	_08: u64,
}

/// SEAD sequence header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct SeqHdr {
	ver: u16,
	_01: u8,
	_size: u16,
}

/// SEAD track
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Track {
	_ver: u8,
	kind: u8,
	_size: u16,
	idx: u16,
	_bank: u16,
	_id: u16,
	_child_id: u16,
	_0c: u8,
}

/// SEAD sequence command header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct SeqCmdHdr {
	_ver: u8,
	size: u8,
	kind: u8,
	body: u8,
}

/// SEAD sequence command track
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct CmdTrack {
	trk_idx: u32,
	_looped: bool,
	_05: u8,
	_trk_id: u16,
	_play_len: f32,
}

/// SEAD material header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct MatHdr {
	_ver: u8,
	_01: u8,
	_size: u16,
	nentries: u16,
	_align: [u8; 12],
}

/// SEAD stream header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct StreamHdr {
	_ver: u8,
	_01: u8,
	_size: u16,
	nchannels: u8,
	codec: u8,
	mat_idx: u16,
	sample_rate: u32,
	loop_start: u32,
	loop_end: u32,
	total_size: u32,
	stream_size: u32,
	_id: u16,
	_align: u16,
}

#[derive(Debug)]
enum SeqVer {
	Old(OldSeq),
	New(NewSeq),
}

impl SeqVer {
	fn new(ver: u8, buf: &mut impl Read) -> Result<SeqVer> {
		match ver {
			v if v <= 2 => {
				let mut seq = OldSeq::zeroed();
				buf.read_exact(bytes_of_mut(&mut seq))?;
				Ok(SeqVer::Old(seq))
			},
			_ => {
				let mut seq = NewSeq::zeroed();
				buf.read_exact(bytes_of_mut(&mut seq))?;
				Ok(SeqVer::New(seq))
			},
		}
	}
}

/// SEAD sequence command
#[derive(Debug)]
struct SeqCmd {
	hdr: SeqCmdHdr,
	cmdtrk: Option<CmdTrack>,
	trk: Option<Track>,
}

impl SeqCmd {
	fn new(buf: &mut impl Read) -> Result<SeqCmd> {
		let mut hdr = SeqCmdHdr::zeroed();
		buf.read_exact(bytes_of_mut(&mut hdr))?;

		let mut cmdtrk = None;
		let mut trk = None
		if hdr.kind == 2 { // key on
			let mut ct = CmdTrack::zeroed();
			let mut t = Track::zeroed();
			buf.read_exact(bytes_of_mut(&mut ct))?;
			buf.read_exact(bytes_of_mut(&mut t))?;
			cmdtrk = Some(ct);
			trk = Some(t);
		}

		Ok(SeqCmd { hdr, cmdtrk, trk })
	}
}

/// SEAD sequence
#[derive(Debug)]
struct Sequence {
	info: SeqInfo,
	hdr: SeqHdr,
	ver: SeqVer,
	cmds: Vec<SeqCmd>,
}

impl Sequence {
	fn new(sead: &SEAD, buf: &mut impl Read) -> Result<Sequence> {
		let mut c = Cursor::new(buf);

		let mut info = SeqInfo::zeroed();
		c.read_exact(bytes_of_mut(&mut info)?;

		if let Some(offs) = sead.chunk_offs.get(ChunkType::Sequences) {
			c.set_position(*offs);

			let mut hdr = SeqHdr::zeroed();
			c.read_exact(bytes_of_mut(&mut hdr)?;

			let ver = SeqVer::new(hdr.ver, &mut c)?;
		}
	}
}

/// SEAD sound section
#[derive(Debug)]
struct Sound {
	hdr: SndHdr,
	name: String,
}

/// SEAD material section
#[derive(Debug)]
struct Material {
	hdr: MatHdr,
	offs: Vec<u32>,
	stream_hdr: StreamHdr,
}

impl Material {
	fn new<T: Buf + Read>(buf: &mut T) -> Result<Material> {
		let mut hdr = MatHdr::zeroed();
		buf.read_exact(bytes_of_mut(&mut hdr))?;

		let offs = (0..hdr.nentries).into_iter().map(|i| buf.get_u32_le()).collect::<Vec<u32>>();

		let mut stream_hdr = StreamHdr::zeroed();
		buf.read_exact(bytes_of_mut(&mut stream_hdr))?;

		Ok(Material { hdr, offs, stream_hdr })
	}
}


#[derive(Debug)]
struct SEAD {
	hdr: Header,
	name: String,
	sect_offs: u32,
	chunk_offs: HashMap<ChunkType, u32>,
	mat: Option<Material>,
}

impl SEAD {
	fn new(buf: &[u8]) -> Result<SEAD> {
		let mut c = Cursor::new(buf);

		let mut hdr = Header::zeroed();
		c.read_exact(bytes_of_mut(&mut hdr))?;

		let mut name = [0; 16];
		c.read_exact(&mut name[..])?;

		let sect_offs = align_size_to_block(16 + hdr.filename_size + 1, 16);

		let chunk_info = vec![ChkTblEntry::zeroed(); hdr.nchunks as usize]
			.iter_mut()
			.for_each(|chk| c.read_exact(bytes_of_mut(chk))?);

		let mut chunk_offs = HashMap::with_capacity(hdr.nchunks as usize)
		chunk_info.iter().for_each(|chk| {
			if ChunkType::from(chk.id) != ChunkType::Unknown {
				chunk_offs.insert(ChunkType::from(chk.id), chk.offs);
			}
		});

		let mat: Option<Material>;
		if let Some(offs) = chunk_offs.get(ChunkType::Materials) {
			c.set_position(*offs as u64);
			mat = Some(Material::new(&mut c)?);
		}

		/*let mut chunks = vec![vec![], hdr.nchunks].enumerate().iter_mut().for_each(|(i, *chk)| {
			chk.resize(chunk_info[i].size as usize, 0);
			c.set_position(chunk_info[i].offs as u64);
			let _ = c.read_exact(&mut chk[..])?;
		});*/

		Ok(SEAD {
			hdr,
			name: String::from_utf8(&name[..].to_vec())?,
			sect_offs,
			chunk_offs,
			mat,
		})
	}
}

const fn align_size_to_block(value: u32, block_align: u32) -> u32 {
	if block_align == 0 {
		return 0;
	}

	let extra_size = value % block_align;
	if extra_size == 0 {
		return value;
	}

	value + block_align - extra_size
}
