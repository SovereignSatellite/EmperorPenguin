use std::{
	fs::File,
	io,
	path::{Path, PathBuf},
};

use super::{WineBox, box_file};

const DOWNLOADS: [&str; 4] = ["drive_c", "users", "Public", "Downloads"];

pub fn import(wine_box: &WineBox, file: &Path) -> Result<PathBuf, String> {
	let name = file.file_name().ok_or("That isn't a file.")?;
	let mut copy = box_file::create_inside(&wine_box.prefix(), DOWNLOADS, name)
		.map_err(|error| error.to_string())?;

	io::copy(
		&mut File::open(file).map_err(|error| error.to_string())?,
		&mut copy,
	)
	.map_err(|error| error.to_string())?;

	Ok(DOWNLOADS.iter().collect::<PathBuf>().join(name))
}
