use bitflags::bitflags;
use bytemuck::{bytes_of_mut,Zeroable};
use bytemuck_derive::{Pod, Zeroable};
use bytes::Buf;
use png::{BitDepth, ColorType, Encoder, EncodingError};

use std::{
    fs::{self, File},
    io::{self, BufWriter, Cursor, Read},
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
    FileWrite(EncodingError),
    FlagsInvalid,
    HeaderRead(io::Error),
    ImageHeaderRead(io::Error),
    IndexRead(io::Error),
    Magic(u32),
    PathWrite,
}

/// TIM image
#[derive(Debug)]
struct Image {
    header: Header,
    data: Vec<u8>,
    bpp: u32,
    w: u16,
    h: u16,
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
            let w = match bpp {
                4 => imgh.w >> 2,
                8 => imgh.w >> 1,
                _ => return Err(TIMErr::BitsPerPixel(bpp)),
            };

            let mut idx = match bpp {
                4 => vec![0; (w * imgh.h >> 1) as usize],
                8 => vec![0; (w * imgh.h) as usize],
                _ => return Err(TIMErr::BitsPerPixel(bpp)),
            };

            c.read_exact(&mut idx[..])
                .map_err(|e| TIMErr::IndexRead(e))?;

            let mut data = Vec::with_capacity((w * imgh.h * 4) as usize);
            for i in idx.iter() {
                match bpp {
                    4 => {
                        let (r, g, b, a) = rgba5551_to_rgba8888(clut[(*i & 240) as usize] as u32);
                        data.push(r);
                        data.push(g);
                        data.push(b);
                        data.push(a);

                        let (r, g, b, a) = rgba5551_to_rgba8888(clut[(*i & 15) as usize] as u32);
                        data.push(r);
                        data.push(g);
                        data.push(b);
                        data.push(a);
                    },
                    8 => {
                        let (r, g, b, a) = rgba5551_to_rgba8888(clut[*i as usize] as u32);
                        data.push(r);
                        data.push(g);
                        data.push(b);
                        data.push(a);
                    },
                    _ => return Err(TIMErr::BitsPerPixel(bpp)),
                }
            }

            Ok(Image {
                header: Header::Indexed(header, imgh),
                data,
                bpp,
                w,
                h: imgh.h,
            })
        } else {
            let mut header = NonIndexedHeader::zeroed();
            c.read_exact(bytes_of_mut(&mut header))
                .map_err(|e| TIMErr::HeaderRead(e))?;

            let npixels = (header.w * header.h) as usize;
            let mut data = Vec::with_capacity(npixels * 4);
            for _ in 0..npixels {
                let (r, g, b, a) = rgba5551_to_rgba8888(c.get_u16_le() as u32);
                data.push(r);
                data.push(g);
                data.push(b);
                data.push(a);
            }

            Ok(Image {
                header: Header::NonIndexed(header),
                data,
                bpp,
                w: header.w,
                h: header.h,
            })
        }
    }

    /// Saves the imported image to a PNG file
    pub fn save(&self, path: &str) -> Result<(), TIMErr> {
        let file = File::create(path).map_err(|_| TIMErr::PathWrite)?;
        let ref mut w = BufWriter::new(file);
        let mut enc = Encoder::new(w, self.w as u32, self.h as u32);

        enc.set_color(ColorType::Rgba);
        enc.set_depth(BitDepth::Eight);

        enc.write_header()
            .map_err(|e| TIMErr::FileWrite(e))?
            .write_image_data(&self.data[..])
            .map_err(|e| TIMErr::FileWrite(e))?;

        Ok(())
    }
}

/// Expands a 5 bit value to a full byte
const fn scale5to8(i: u8) -> u8 {
    (i << 3) | (i >> 2)
}

/// Converts a colour value from RGBA5551 to RGBA8888
const fn rgba5551_to_rgba8888(i: u32) -> (u8, u8, u8, u8) {
    let r = scale5to8((i & 31) as u8);
    let g = scale5to8(((i >> 5) & 31) as u8);
    let b = scale5to8(((i >> 10) & 31) as u8);
    let a = !scale5to8(((i >> 15) & 31) as u8);

    (r, g, b, a)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tim_import() {
        let img = super::Image::load("/home/admin/Documents/GitHub/KajarEngine/test data/0025.tim");
        println!("{:?}", img);
    }

    #[test]
    fn test_tim_export() {
        let img = super::Image::load("/home/admin/Documents/GitHub/KajarEngine/test data/0025.tim").unwrap();
        img.save("0025.png").unwrap();
    }
}
