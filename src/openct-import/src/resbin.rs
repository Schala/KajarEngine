// Credit to https://github.com/jimzrt/ChronoMod

use blowfish::BlowfishLE;
use bytemuck::{bytes_of_mut, Zeroable};
use bytemuck_derive::{Pod, Zeroable};
use bytes::Buf;
use cipher::{BlockDecrypt, KeyInit};

use libz_sys::{
    inflate, inflateEnd, inflateInit2_, uInt, z_stream, zlibVersion, Bytef, Z_FINISH, Z_OK,
    Z_STREAM_END,
};

use std::{
    collections::HashMap,
    ffi::c_int,
    fs,
    io::{self, Cursor, Read},
    mem::{size_of, MaybeUninit},
    path::PathBuf,
    ptr::addr_of_mut,
};

use crate::{read_cstr, tag};

const KEY_OFFSET: u64 = 0x398EE8;

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
    entries: HashMap<PathBuf, (ResEntry, Vec<u8>)>,
    cipher: BlowfishLE,
}

#[derive(Debug)]
pub enum ResBinErr {
    CipherInit,
    CmpRead(io::Error),
    Decmp(c_int),
    Dump(io::Error),
    EntryDataRead(PathBuf, io::Error),
    EntryPath(PathBuf),
    EntryRead(io::Error),
    ExeRead(io::Error),
    FileRead(PathBuf, io::Error),
    HeaderMismatch(u32),
    HeaderRead(io::Error),
    KeyRead(io::Error),
    PathName(ResEntry, io::Error),
}

impl ResBin {
    /// Loads all data from resources.bin
    pub fn load(filepath: &str, ctexe: &str) -> Result<ResBin, ResBinErr> {
        // decryption key
        let mut key = [0; 16];
        let mut exe = Cursor::new(fs::read(ctexe).map_err(|e| ResBinErr::ExeRead(e))?);
        exe.set_position(KEY_OFFSET);
        exe.read_exact(&mut key[..])
            .map_err(|e| ResBinErr::KeyRead(e))?;

        // buffer file
        let buf =
            fs::read(filepath).map_err(|e| ResBinErr::FileRead(PathBuf::from(filepath), e))?;

        let mut header = Header::zeroed();
        let mut fc = Cursor::new(buf);

        // header
        fc.read_exact(bytes_of_mut(&mut header))
            .map_err(|e| ResBinErr::HeaderRead(e))?;

        decode(0, bytes_of_mut(&mut header));

        if header.sig != tag!(b"ARC1") {
            return Err(ResBinErr::HeaderMismatch(header.sig));
        }

        // compressed data
        let mut cmp = vec![0; header.cmp_size as usize];
        fc.set_position(header.offs as u64);
        fc.read_exact(&mut cmp[..])
            .map_err(|e| ResBinErr::CmpRead(e))?;

        decode(header.offs, &mut cmp[..]);
        let dcmp = decompress(&mut cmp[4..], header.size as usize)?;

        // decompressed data
        let mut dc = Cursor::new(&dcmp[..]);
        let n = dc.get_u32_le();
        let mut entdata = vec![ResEntry::zeroed(); n as usize];

        for ent in entdata.iter_mut() {
            dc.read_exact(bytes_of_mut(ent))
                .map_err(|e| ResBinErr::EntryRead(e))?;
        }

        // entries
        let mut entries = HashMap::with_capacity(n as usize);
        for ent in entdata.iter() {
            dc.set_position(ent.path_offs as u64);

            let s = read_cstr(&mut dc).map_err(|e| ResBinErr::PathName(ent.clone(), e))?;
            let path = PathBuf::from(s);
            let mut cdata = vec![0; ent.size as usize];

            fc.set_position(ent.data_offs as u64);
            fc.read_exact(&mut cdata[..])
                .map_err(|e| ResBinErr::EntryDataRead(path.clone(), e))?;

            decode(ent.data_offs, &mut cdata);
            let size = get_u32_le(&cdata[..]) as usize;
            let ddata = decompress(&mut cdata[4..], size)?;

            entries.insert(path, (*ent, ddata));
        }

        Ok(ResBin {
            header,
            entries,
            cipher: BlowfishLE::new_from_slice(&key).map_err(|_| ResBinErr::CipherInit)?,
        })
    }

    /// Decrypts a single file entry
    pub fn decrypt(&mut self, path: &str) -> Result<(), ResBinErr> {
        let (info, data) = self
            .entries
            .get_mut(&PathBuf::from(path))
            .ok_or(ResBinErr::EntryPath(PathBuf::from(path)))?;

        data[0] ^= 0x75;
        data[1] ^= 0xFA;
        data[2] ^= 0x29;
        data[3] ^= 0x95;
        data[4] ^= 0x05;
        data[5] ^= 0x4D;
        data[6] ^= 0x41;
        data[7] ^= 0x5F;

        let mut ddata = vec![0; info.size as usize];
        for (block_in, block_out) in data.chunks(8).zip(ddata.chunks_mut(8)) {
            self.cipher
                .decrypt_block_b2b(block_in.into(), block_out.into());
        }

        Ok(())
    }

    /// Dumps the contents of a single entry to file.
    pub fn dump(&self, in_path: &str, out_path: &str) -> Result<(), ResBinErr> {
        let (_, ent) = self
            .entries
            .get(&PathBuf::from(in_path))
            .ok_or(ResBinErr::EntryPath(PathBuf::from(in_path)))?;
        let mut path = PathBuf::from(out_path);
        path.push(in_path);

        fs::write(path.as_path(), &ent[..]).map_err(|e| ResBinErr::Dump(e))?;

        Ok(())
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
    #[test]
    fn test_resbin_extract() {
        let mut resb = super::ResBin::load(
            "/Users/roymeurin/Desktop/resources.bin",
            "/Users/roymeurin/Downloads/Chrono Trigger.exe",
        )
        .unwrap();
        resb.decrypt("string_1.bin").unwrap();
        resb.dump("string_1.bin", ".").unwrap();
        //assert_eq(resb.is_ok());
    }
}
