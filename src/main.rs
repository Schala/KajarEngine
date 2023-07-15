use anyhow::Result;

mod resbin;
use resbin::*;

mod util;

use std::{
    env,
    fs::read
};

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let buf = read(&args[1])?;
    let res = ResBin::new(&buf[..]);

    println!("{:?}", res);
    Ok(())
}
