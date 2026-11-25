use std::{fs::File, os::unix::fs::FileExt as _, path::Path};

use super::bytes::{array, read_file, u16_at, u32_at};

use crate::boxes::open_regular;

const ICON_BUDGET: usize = 4 << 20;
/// A 256-pixel image in 32-bit color with its mask, the largest an icon holds uncompressed.
const LARGEST_IMAGE: usize = 40 + 256 * 256 * 4 + 256 * 256 / 8;
const LARGEST_SIDE: u32 = 256;
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

const ICON: u32 = 3;
const GROUP_ICON: u32 = 14;
const SUBDIRECTORY: u32 = 0x8000_0000;
const GROUP_ENTRY_SIZE: usize = 14;
const FILE_ENTRY_SIZE: usize = 16;

struct Reader {
	file: File,
	budget: usize,
}

impl Reader {
	fn append(&mut self, buffer: &mut Vec<u8>, offset: u64, length: usize) -> Option<()> {
		self.budget = self.budget.checked_sub(length)?;

		let start = buffer.len();

		buffer.resize(start + length, 0);

		self.file.read_exact_at(&mut buffer[start..], offset).ok()
	}

	fn read(&mut self, offset: u64, length: usize) -> Option<Vec<u8>> {
		let mut bytes = Vec::new();

		self.append(&mut bytes, offset, length)?;

		Some(bytes)
	}
}

struct Section {
	address: u32,
	size: u32,
	file_offset: u32,
}

struct Resources {
	reader: Reader,
	sections: Vec<Section>,
	root: u64,
}

impl Section {
	fn parse(header: &[u8; 40]) -> Option<Self> {
		Some(Self {
			address: u32_at(header, 12)?,
			size: u32_at(header, 8)?.max(u32_at(header, 16)?),
			file_offset: u32_at(header, 20)?,
		})
	}
}

const fn entry_fields([name, offset]: [[u8; 4]; 2]) -> (u32, u32) {
	(u32::from_le_bytes(name), u32::from_le_bytes(offset))
}

fn file_offset(sections: &[Section], address: u32) -> Option<u64> {
	let section = sections
		.iter()
		.find(|section| address >= section.address && address - section.address < section.size)?;

	Some(u64::from(section.file_offset) + u64::from(address - section.address))
}

fn subdirectory(entries: &[(u32, u32)], id: u32) -> Option<u32> {
	let &(_, offset) = entries.iter().find(|&&(name, _)| name == id)?;

	(offset & SUBDIRECTORY != 0).then_some(offset)
}

impl Resources {
	fn read(mut reader: Reader) -> Option<Self> {
		let pe = u64::from(u32_at(&reader.read(0, 64)?, 0x3C)?);
		let headers = reader.read(pe, 24)?;

		if headers[..4] != *b"PE\0\0" {
			return None;
		}

		let optional_size = usize::from(u16_at(&headers, 20)?);
		let optional = reader.read(pe + 24, optional_size)?;
		let directories = match u16_at(&optional, 0)? {
			0x10b => 96,
			0x20b => 112,
			_ => return None,
		};
		let resource_address = u32_at(&optional, directories + 2 * 8)?;
		let section_count = usize::from(u16_at(&headers, 6)?);
		let table = reader.read(pe + 24 + optional_size as u64, section_count * 40)?;
		let (section_headers, _) = table.as_chunks::<40>();
		let sections: Vec<Section> = section_headers
			.iter()
			.map(Section::parse)
			.collect::<Option<_>>()?;

		Some(Self {
			reader,
			root: file_offset(&sections, resource_address)?,
			sections,
		})
	}

	/// Each entry pairs a name or ID with the offset of a subdirectory or data entry.
	fn entries(&mut self, directory: u32) -> Option<Vec<(u32, u32)>> {
		let offset = self.root + u64::from(directory & !SUBDIRECTORY);
		let header = self.reader.read(offset, 16)?;
		let count = usize::from(u16_at(&header, 12)?) + usize::from(u16_at(&header, 14)?);
		let table = self.reader.read(offset + 16, count * 8)?;
		let (words, _) = table.as_chunks::<4>();
		let (entries, _) = words.as_chunks::<2>();

		Some(entries.iter().copied().map(entry_fields).collect())
	}

	/// Takes the first language of a resource.
	fn append_data(&mut self, resource: u32, buffer: &mut Vec<u8>) -> Option<()> {
		let (_, entry) = *self.entries(resource)?.first()?;
		let header = self.reader.read(self.root + u64::from(entry), 8)?;
		let offset = file_offset(&self.sections, u32_at(&header, 0)?)?;
		let length = usize::try_from(u32_at(&header, 4)?).ok()?;

		self.reader.append(buffer, offset, length)
	}

	/// Counts from the first group for a non-negative index, and names a group ID otherwise.
	fn group(&mut self, groups: u32, index: i32) -> Option<Vec<u8>> {
		let entries = self.entries(groups)?;
		let (_, offset) = if let Ok(position) = usize::try_from(index) {
			*entries.get(position)?
		} else {
			let id = index.checked_neg()?.cast_unsigned();

			*entries.iter().find(|&&(name, _)| name == id)?
		};
		let mut group = Vec::new();

		self.append_data(offset, &mut group)?;

		Some(group)
	}
}

fn side(byte: u8) -> u32 {
	if byte == 0 {
		LARGEST_SIDE
	} else {
		u32::from(byte)
	}
}

/// Icon files and icon groups start each directory entry with the same fields: width, height, colors,
/// a reserved byte, planes, and bits per pixel.
fn rank(entry: &&[u8]) -> (u32, Option<u16>) {
	(side(entry[0]), u16_at(entry, 6))
}

fn best(directory: &[u8], entry_size: usize) -> Option<&[u8]> {
	let count = usize::from(u16_at(directory, 4)?);

	directory
		.get(6..)?
		.chunks_exact(entry_size)
		.take(count)
		.max_by_key(rank)
}

/// Decoders size an image by its own header, whatever its directory entry says, and a bitmap's
/// height counts its mask too.
fn size(image: &[u8]) -> Option<(u32, u32)> {
	if image.starts_with(PNG_SIGNATURE) {
		return Some((
			array(image, 16).map(u32::from_be_bytes)?,
			array(image, 20).map(u32::from_be_bytes)?,
		));
	}

	Some((u32_at(image, 4)?, u32_at(image, 8)? / 2))
}

fn drawable((width, height): (u32, u32)) -> bool {
	(1..=LARGEST_SIDE).contains(&width) && (1..=LARGEST_SIDE).contains(&height)
}

fn fits(image: &[u8]) -> bool {
	image.len() <= LARGEST_IMAGE && size(image).is_some_and(drawable)
}

fn single_icon(entry: &[u8], image: &[u8]) -> Option<Vec<u8>> {
	if !fits(image) {
		return None;
	}

	let mut file = Vec::with_capacity(22 + image.len());

	file.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
	file.extend_from_slice(&entry[..8]);
	file.extend_from_slice(&(image.len() as u32).to_le_bytes());
	file.extend_from_slice(&22_u32.to_le_bytes());
	file.extend_from_slice(image);

	Some(file)
}

fn from_icon_file(path: &Path) -> Option<Vec<u8>> {
	let file = read_file(path, ICON_BUDGET as u64)?;
	let entry = best(&file, FILE_ENTRY_SIZE)?;
	let length = u32_at(entry, 8)? as usize;
	let offset = u32_at(entry, 12)? as usize;

	single_icon(entry, file.get(offset..offset.checked_add(length)?)?)
}

fn from_executable(path: &Path, index: i32) -> Option<Vec<u8>> {
	let reader = Reader {
		file: open_regular(path).ok()?,
		budget: ICON_BUDGET,
	};
	let mut resources = Resources::read(reader)?;
	let root = resources.entries(0)?;
	let group = resources.group(subdirectory(&root, GROUP_ICON)?, index)?;
	let entry = best(&group, GROUP_ENTRY_SIZE)?;
	let icons = resources.entries(subdirectory(&root, ICON)?)?;
	let mut image = Vec::new();

	resources.append_data(
		subdirectory(&icons, u32::from(u16_at(entry, 12)?))?,
		&mut image,
	)?;

	single_icon(entry, &image)
}

pub fn extract(path: &Path, index: i32) -> Option<Vec<u8>> {
	let is_icon_file = path
		.extension()
		.is_some_and(|extension| extension.eq_ignore_ascii_case("ico"));

	if is_icon_file {
		return from_icon_file(path);
	}

	from_executable(path, index)
}
