use core::ops::Range;

use super::bytes::{array, u16_at, u32_at};

const HEADER_SIZE: u32 = 0x4C;
const HAS_TARGET_ID_LIST: u32 = 0x01;
const HAS_LINK_INFO: u32 = 0x02;
const HAS_ICON_LOCATION: u32 = 0x40;
const IS_UNICODE: u32 = 0x80;
/// The flags of the strings after the link info, in the order they follow it.
const STRING_FLAGS: [u32; 5] = [0x04, 0x08, 0x10, 0x20, HAS_ICON_LOCATION];
const HAS_LOCAL_BASE_PATH: u32 = 0x01;
/// Link info headers at least this long also hold their paths in UTF-16.
const UNICODE_LINK_INFO_SIZE: u32 = 0x24;

pub struct Shortcut {
	pub target: String,
	pub icon_location: Option<String>,
	pub icon_index: i32,
}

/// Windows' ANSI code page 1252 differs from Latin-1 in 0x80 to 0x9F, which this rejects.
fn ansi(bytes: &[u8]) -> Option<String> {
	bytes
		.iter()
		.map(|&byte| (!(0x80..0xA0).contains(&byte)).then_some(char::from(byte)))
		.collect()
}

fn utf16(bytes: &[u8]) -> Option<String> {
	let (pairs, _) = bytes.as_chunks::<2>();
	let units: Vec<u16> = pairs.iter().map(|&pair| u16::from_le_bytes(pair)).collect();

	String::from_utf16(&units).ok()
}

fn ansi_until_nul(bytes: &[u8], offset: u32) -> Option<String> {
	let rest = bytes.get(usize::try_from(offset).ok()?..)?;

	ansi(&rest[..rest.iter().position(|&byte| byte == 0)?])
}

fn utf16_until_nul(bytes: &[u8], offset: u32) -> Option<String> {
	let rest = bytes.get(usize::try_from(offset).ok()?..)?;
	let end = rest
		.as_chunks::<2>()
		.0
		.iter()
		.position(|pair| *pair == [0, 0])?;

	utf16(&rest[..end * 2])
}

fn local_path(info: &[u8]) -> Option<String> {
	if u32_at(info, 8)? & HAS_LOCAL_BASE_PATH == 0 {
		return None;
	}

	let (mut path, suffix) = if u32_at(info, 4)? >= UNICODE_LINK_INFO_SIZE {
		(
			utf16_until_nul(info, u32_at(info, 0x1C)?)?,
			utf16_until_nul(info, u32_at(info, 0x20)?)?,
		)
	} else {
		(
			ansi_until_nul(info, u32_at(info, 0x10)?)?,
			ansi_until_nul(info, u32_at(info, 0x18)?)?,
		)
	};

	path.push_str(&suffix);

	Some(path)
}

fn string_range(bytes: &[u8], offset: usize, unicode: bool) -> Option<Range<usize>> {
	let characters = usize::from(u16_at(bytes, offset)?);
	let start = offset + 2;

	Some(start..start + characters * if unicode { 2 } else { 1 })
}

impl Shortcut {
	/// Reads shortcuts to local paths in the Shell Link format ([MS-SHLLINK]).
	pub fn parse(bytes: &[u8]) -> Option<Self> {
		if u32_at(bytes, 0)? != HEADER_SIZE {
			return None;
		}

		let flags = u32_at(bytes, 0x14)?;
		let icon_index = i32::from_le_bytes(array(bytes, 0x38)?);
		let mut offset = HEADER_SIZE as usize;

		if flags & HAS_TARGET_ID_LIST != 0 {
			offset += 2 + usize::from(u16_at(bytes, offset)?);
		}
		if flags & HAS_LINK_INFO == 0 {
			return None;
		}

		let info_end = offset.checked_add(usize::try_from(u32_at(bytes, offset)?).ok()?)?;
		let target = local_path(bytes.get(offset..info_end)?)?;
		let unicode = flags & IS_UNICODE != 0;
		let mut icon_location = None;

		offset = info_end;

		for flag in STRING_FLAGS.into_iter().filter(|flag| flags & flag != 0) {
			let range = string_range(bytes, offset, unicode)?;

			offset = range.end;

			if flag == HAS_ICON_LOCATION {
				let text = bytes.get(range)?;
				let mut location = if unicode { utf16(text) } else { ansi(text) }?;

				// Some installers count a terminating NUL that the format does not have.
				location.truncate(location.find('\0').unwrap_or(location.len()));

				icon_location = Some(location);
			}
		}

		Some(Self {
			target,
			icon_location,
			icon_index,
		})
	}
}
