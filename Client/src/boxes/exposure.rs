use std::{
	fs,
	path::{Path, PathBuf},
};

use super::{Access, Drive, Folder, list};

fn real_path(folder: &Folder) -> Option<PathBuf> {
	fs::canonicalize(folder.path()).ok()
}

fn listed_box(line: &str) -> Option<PathBuf> {
	real_path(&Folder::new(Path::new(line)).ok()?)
}

/// Programs in a box that can write a folder holding a box's settings could change what that box
/// reaches when it next starts.
pub fn holds_a_box(folder: &Folder) -> Result<bool, String> {
	let Some(shared) = real_path(folder) else {
		return Ok(false);
	};

	Ok(list::read()?
		.lines()
		.filter_map(listed_box)
		.any(|listed| listed.starts_with(&shared)))
}

pub fn check_drive(drive: &Drive) -> Result<(), String> {
	if drive.access == Access::ReadWrite && holds_a_box(&drive.folder)? {
		return Err(format!(
			"{} holds a box, so boxes can only read it.",
			drive.folder.text()
		));
	}

	Ok(())
}
