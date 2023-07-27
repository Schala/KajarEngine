use anyhow::Result;
use lazy_static::lazy_static;
use murmurhash32::murmurhash3;
use std::collections::HashMap;

use crate::markup::ident_array;

lazy_static! {
	static ref DLG_FILES: HashMap<&'static str, usize> = {
		let mut m = HashMap::new();
		m.insert("cmes", 6);
		m.insert("comu", 1);
		m.insert("exms", 4);
		m.insert("kmes", 3);
		m.insert("mesi", 1);
		m.insert("mesk", 5);
		m.insert("mess", 1);
		m.insert("mest", 6);
		m.insert("mon_tec", 1);
	};
}

pub fn import_dialogue(path: &str) -> Result<()> {
}
