#[cfg(feature = "cc_psx")]
mod atim;

#[cfg(feature = "cc_psx")]
mod drp;

#[cfg(feature = "cc_psx")]
mod tim;


#[cfg(feature = "cc_psx")]
use atim;

#[cfg(feature = "cc_psx")]
use drp;

#[cfg(feature = "cc_psx")]
use tim;


/// CPT errors
#[cfg(feature = "cc_psx")]
#[derive(Debug)]
pub enum CPTErr {
    ArchiveRead(io::Error),
    ChildRead(io::Error),
}

/// Loads the inner files of a .cpt file from the specified path
#[cfg(feature = "cc_psx")]
pub fn load_cpt(path: &str) -> Result<Vec<Vec<u8>>, CPTErr> {
    let cpt = fs::read(path).map_err(|e| CPTErr::ArchiveRead(e))?;
    let n = cpt.get_u32_le() as usize;
    let ptrs = (0..n)
        .iter()
        .map(|_| cpt.get_u32_le() as usize)
        .collect::<Vec<usize>>();
    let has_eof = ptrs[n] == cpt.len();

    let files = if has_eof {
        (0..(n - 1))
            .iter()
            .map(|i| {
                let mut bin = vec![0; ptrs[i + 1] - ptrs[i]];
                cpt.read_exact(&mut bin[..])
                    .map_err(|e| CPTErr::ChildRead(e))?;
                bin
            })
            .collect::<Vec<Vec<u8>>>();
    } else {
        (0..n)
            .iter()
            .map(|i| {
                let mut bin = if i == n {
                    vec![0; cpt.len() - ptrs[i]]
                } else {
                    vec![0; ptrs[i + 1] - ptrs[i]]
                };

                cpt.read_exact(&mut bin[..])
                    .map_err(|e| CPTErr::ChildRead(e))?;
                bin
            })
            .collect::<Vec<Vec<u8>>>();
    };

    Ok(files)
}
