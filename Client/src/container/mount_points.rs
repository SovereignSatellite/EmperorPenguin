use std::fs;

use crate::boxes::{DRIVE_LETTERS, WineBox};

/// Podman creates a mount point for each drive, and one left in the prefix shows as an empty drive.
pub fn remove_unused(wine_box: &WineBox) {
	let drives = &wine_box.settings.drives;

	for letter in DRIVE_LETTERS {
		if drives.iter().all(|drive| drive.letter != letter) {
			let mount_point = wine_box.prefix().join(format!("dosdevices/{letter}:"));

			fs::remove_dir(mount_point).unwrap_or_default();
		}
	}
}
