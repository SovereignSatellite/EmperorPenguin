use std::{io::Read as _, path::Path};

use crate::boxes::open_regular;

pub fn array<const N: usize>(bytes: &[u8], offset: usize) -> Option<[u8; N]> {
	bytes.get(offset..offset.checked_add(N)?)?.try_into().ok()
}

pub fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
	array(bytes, offset).map(u16::from_le_bytes)
}

pub fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
	array(bytes, offset).map(u32::from_le_bytes)
}

pub fn read_file(path: &Path, limit: u64) -> Option<Vec<u8>> {
	let mut bytes = Vec::new();

	open_regular(path)
		.ok()?
		.take(limit + 1)
		.read_to_end(&mut bytes)
		.ok()?;

	(bytes.len() as u64 <= limit).then_some(bytes)
}
