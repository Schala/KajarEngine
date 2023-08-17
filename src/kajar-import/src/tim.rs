use bitflags::bitflags;
use bytemuck::{bytes_of_mut, Zeroable};
use bytemuck_derive::{Pod, Zeroable};
use bytes::Buf;

use std::{
    fs,
    io::{self, Cursor, Read},
};

bitflags! {
    /// Image attributes
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    #[repr(C)]
    pub struct Flags: u32 {
        const BPP_4 = 0;
        const BPP_8 = 1;
        const BPP_16 = 2;
        const BPP_24 = 3;
        const MIXED = 4;
        const INDEXED = 8;
    }
}

/// Indexed TIM file header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct IndexedHeader {
    magic: u32,
    flags: Flags,
    clut_size: u32,
    _palx: u16,
    _paly: u16,
    ncolors: u16,
    ncluts: u16,
}

/// Indexed image header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct IndexedImageHeader {
    size: u32,
    addr_x: u16,
    addr_y: u16,
    w: u16,
    h: u16,
}

/// Non-indexed TIM file header
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct NonIndexedHeader {
    magic: u32,
    flags: Flags,
    size: u32,
    addr_x: u16,
    addr_y: u16,
    w: u16,
    h: u16,
}

/// TIM file header
#[derive(Debug)]
enum Header {
    Indexed(IndexedHeader, IndexedImageHeader),
    NonIndexed(NonIndexedHeader),
}

/// TIM image import error
#[derive(Debug)]
pub enum TIMErr {
    BitsPerPixel(u32),
    FileRead(io::Error),
    FlagsInvalid,
    HeaderRead(io::Error),
    ImageHeaderRead(io::Error),
    IndexRead(io::Error),
    Magic(u32),
}

/// TIM image
#[derive(Debug)]
struct Image {
    header: Header,
    pixels: Vec<u32>,
    bpp: u32,
}

impl Image {
    /// Loads in a TIM format image file
    pub fn load(path: &str) -> Result<Image, TIMErr> {
        let mut c = Cursor::new(fs::read(path).map_err(|e| TIMErr::FileRead(e))?);

        let magic = c.get_u32_le();
        if magic != 16 {
            return Err(TIMErr::Magic(magic));
        }

        let flags = Flags::from_bits(c.get_u32_le()).ok_or(TIMErr::FlagsInvalid)?;
        let bpp = if (flags.bits() & 6) != 0 {
            (flags.bits() & 6) << 3
        } else {
            4
        };

        c.set_position(0);
        if flags.contains(Flags::INDEXED) {
            let mut header = IndexedHeader::zeroed();
            c.read_exact(bytes_of_mut(&mut header))
                .map_err(|e| TIMErr::HeaderRead(e))?;

            let mut clut = Vec::with_capacity((header.ncolors * header.ncluts) as usize);
            for _ in 0..clut.len() {
                clut.push(c.get_u16_le());
            }

            let mut imgh = IndexedImageHeader::zeroed();
            c.read_exact(bytes_of_mut(&mut imgh))
                .map_err(|e| TIMErr::ImageHeaderRead(e))?;
            let real_width = match bpp {
                4 => imgh.w >> 2,
                8 => imgh.w >> 1,
                _ => return Err(TIMErr::BitsPerPixel(bpp)),
            };

            let mut idx = match bpp {
                4 => vec![0; (real_width * imgh.h >> 1) as usize],
                8 => vec![0; (real_width * imgh.h) as usize],
                _ => return Err(TIMErr::BitsPerPixel(bpp)),
            };

            c.read_exact(&mut idx[..])
                .map_err(|e| TIMErr::IndexRead(e))?;

            let mut pixels = Vec::with_capacity((real_width * imgh.h) as usize);
            for i in idx.iter() {
                match bpp {
                    4 => {
                        pixels.push(rgba5551_to_argb32(clut[(*i & 240) as usize] as u32));
                        pixels.push(rgba5551_to_argb32(clut[(*i & 15) as usize] as u32));
                    }
                    8 => pixels.push(rgba5551_to_argb32(clut[*i as usize] as u32)),
                    _ => return Err(TIMErr::BitsPerPixel(bpp)),
                }
            }

            Ok(Image {
                header: Header::Indexed(header, imgh),
                pixels,
                bpp,
            })
        } else {
            let mut header = NonIndexedHeader::zeroed();
            c.read_exact(bytes_of_mut(&mut header))
                .map_err(|e| TIMErr::HeaderRead(e))?;

            let npixels = (header.w * header.h) as usize;
            let mut pixels = Vec::with_capacity(npixels);
            for _ in 0..npixels {
                pixels.push(rgba5551_to_argb32(c.get_u16_le() as u32));
            }

            Ok(Image {
                header: Header::NonIndexed(header),
                pixels,
                bpp,
            })
        }
    }
}

/// Expands a 5 bit value to a full byte
const fn scale5to8(i: u32) -> u32 {
    (i << 3) | (i >> 2)
}

/// Converts a colour value from RGBA5551 to ARGB32
const fn rgba5551_to_argb32(i: u32) -> u32 {
    let r = scale5to8(i & 31);
    let g = scale5to8((i >> 5) & 31);
    let b = scale5to8((i >> 10) & 31);
    let a = !scale5to8((i >> 15) & 31);

    (a << 24) | (r << 16) | (g << 8) | b
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tim_import() {
        let img = super::Image::load("/home/admin/Documents/GitHub/KajarEngine/test data/0025.tim");
        println!("{:?}", img);
    }
}
