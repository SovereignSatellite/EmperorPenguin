use std::{
	fs::{self, DirEntry},
	io,
	os::unix::fs::symlink,
	path::Path,
};

use super::host;

use crate::{
	boxes::{WineBox, real_folder},
	folders,
};

const TRANSLATION: &str = env!("EMPEROR_PENGUIN_TRANSLATION");

const NVIDIA_WINE_LIBRARIES: &str = "/run/opengl-driver/lib/nvidia/wine";

fn link_entry(entry: io::Result<DirEntry>, folder: &Path) -> Result<(), String> {
	let entry = entry.map_err(|error| error.to_string())?;
	let (target, link) = (entry.path(), folder.join(entry.file_name()));

	if fs::read_link(&link).is_ok_and(|current| current == target) {
		return Ok(());
	}

	folders::remove_if_present(&link).map_err(|error| error.to_string())?;

	symlink(&target, &link).map_err(|error| error.to_string())
}

fn link_folder(source: &Path, folder: &Path) -> Result<(), String> {
	fs::read_dir(source)
		.map_err(|error| error.to_string())?
		.try_for_each(|entry| link_entry(entry, folder))
}

/// Wine keeps native DLLs that are already in place when it creates or updates the prefix. The links
/// point into the store and the host driver, which the container mounts at the same paths.
pub fn install(wine_box: &WineBox) -> Result<(), String> {
	let prefix = wine_box.prefix();
	let translation = Path::new(TRANSLATION);
	let system = real_folder(&prefix, ["drive_c", "windows", "system32"])?;

	link_folder(&translation.join("system32"), &system)?;
	link_folder(
		&translation.join("syswow64"),
		&real_folder(&prefix, ["drive_c", "windows", "syswow64"])?,
	)?;

	let nvidia = Path::new(NVIDIA_WINE_LIBRARIES);

	if host::has_nvidia_toolkit() && nvidia.is_dir() {
		link_folder(nvidia, &system)?;
	}

	Ok(())
}
