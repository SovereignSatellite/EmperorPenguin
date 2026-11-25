use std::{fs, io, path::Path};

use super::{Folder, WineBox};

use crate::folders;

/// Stays in the list so that an unmounted drive does not drop the box.
pub struct Unreadable {
	pub line: String,
	pub error: String,
}

impl Unreadable {
	pub fn name(&self) -> &str {
		self.line
			.rsplit_once('/')
			.map_or(&self.line, |(_, name)| name)
	}
}

fn read_listed(line: &str) -> Result<WineBox, String> {
	Folder::new(Path::new(line)).and_then(WineBox::read)
}

pub fn read() -> Result<String, String> {
	let list = folders::configuration()?.join("boxes");

	match fs::read_to_string(&list) {
		Ok(text) => Ok(text),
		Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
		Err(error) => Err(format!("{}: {error}", list.display())),
	}
}

pub fn load() -> Result<(Vec<WineBox>, Vec<Unreadable>), String> {
	let text = read()?;
	let mut boxes = Vec::new();
	let mut unreadable = Vec::new();

	for line in text.lines() {
		match read_listed(line) {
			Ok(wine_box) => boxes.push(wine_box),
			Err(error) => unreadable.push(Unreadable {
				line: line.to_owned(),
				error,
			}),
		}
	}

	Ok((boxes, unreadable))
}

fn append_line(mut text: String, line: &str) -> String {
	text.push_str(line);
	text.push('\n');

	text
}

pub fn save<'line, I>(lines: I) -> Result<(), String>
where
	I: Iterator<Item = &'line str>,
{
	let folder = folders::configuration()?;

	fs::create_dir_all(&folder).map_err(|error| error.to_string())?;

	fs::write(folder.join("boxes"), lines.fold(String::new(), append_line))
		.map_err(|error| error.to_string())
}
