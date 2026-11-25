use std::{
	fs, io,
	os::unix::fs::{MetadataExt as _, PermissionsExt as _},
	path::Path,
};

use crate::{boxes::WineBox, folders};

const WINE: &str = env!("EMPEROR_PENGUIN_WINE");

/// Every file in the Nix store has this modification time, and Wine's copies keep it.
const STORE_TIME: i64 = 1;
const OWNER_WRITE: u32 = 0o200;
/// Wine copies its data files into `windows\<folder>` and `windows\<folder>\<folder>`.
const DEEPEST_COPY: usize = 2;

/// Wine's copies of store files keep the store's read-only mode, which stops a later Wine from
/// replacing them. Programs in the box can write these folders, so links are left alone.
fn make_copies_writable(folder: &Path, depth: usize) -> io::Result<()> {
	for entry in fs::read_dir(folder)? {
		let entry = entry?;
		let metadata = entry.metadata()?;

		if metadata.is_dir() && depth > 0 {
			make_copies_writable(&entry.path(), depth - 1)?;
		} else if metadata.is_file()
			&& metadata.mtime() == STORE_TIME
			&& metadata.mode() & OWNER_WRITE == 0
		{
			let writable = fs::Permissions::from_mode(metadata.mode() | OWNER_WRITE);

			fs::set_permissions(entry.path(), writable)?;
		}
	}

	Ok(())
}

/// Wine sets a prefix up again only when `wine.inf` changes its modification time, which every file
/// in the Nix store shares, or when the prefix has no timestamp. So a box records the Wine build it
/// last ran, outside the prefix that its programs can write.
pub fn refresh_prefix(wine_box: &WineBox) -> Result<(), String> {
	let record = wine_box.folder.path().join("wine-build");

	if fs::read_to_string(&record).is_ok_and(|built| built == WINE) {
		return Ok(());
	}

	let prefix = wine_box.prefix();

	make_copies_writable(&prefix.join("drive_c/windows"), DEEPEST_COPY)
		.map_err(|error| error.to_string())?;
	folders::remove_if_present(&prefix.join(".update-timestamp"))
		.map_err(|error| error.to_string())?;

	fs::write(record, WINE).map_err(|error| error.to_string())
}
