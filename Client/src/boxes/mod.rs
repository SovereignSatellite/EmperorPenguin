mod box_file;
mod exposure;
mod folder;
mod import;
mod list;
mod settings;
mod usage;

use std::{
	fs, io,
	os::unix,
	path::{Path, PathBuf},
};

pub use box_file::{create_regular, open_regular, real_folder};
pub use exposure::{check_drive, holds_a_box};
pub use folder::Folder;
pub use import::import;
pub use list::{Unreadable, load, save};
pub use settings::{Access, DRIVE_LETTERS, Drive, Settings, Switch};
pub use usage::disk_usage;

#[derive(Clone)]
pub struct WineBox {
	pub folder: Folder,
	pub settings: Settings,
}

fn settings_file(folder: &Folder) -> PathBuf {
	folder.path().join("settings")
}

fn unreadable(error: &io::Error) -> String {
	if error.kind() == io::ErrorKind::NotFound {
		return "The folder or its settings file is missing.".into();
	}

	error.to_string()
}

fn create_folder(path: &Path) -> Result<(), String> {
	fs::create_dir_all(path).map_err(|error| error.to_string())
}

impl WineBox {
	pub fn read(folder: Folder) -> Result<Self, String> {
		let text =
			fs::read_to_string(settings_file(&folder)).map_err(|error| unreadable(&error))?;
		let settings = Settings::parse(&text)?;

		Ok(Self { folder, settings })
	}

	pub fn prefix(&self) -> PathBuf {
		self.folder.path().join("prefix")
	}

	pub fn save(&self) -> Result<(), String> {
		fs::write(settings_file(&self.folder), self.settings.serialize())
			.map_err(|error| error.to_string())
	}

	/// A new prefix gets its `dosdevices` folder from us, so Wine never adds its `Z:` drive. It also gets its user
	/// registry, whose values Wine keeps when it sets the prefix up.
	pub fn open(folder: Folder) -> Result<Self, String> {
		if settings_file(&folder).exists() {
			return Self::read(folder);
		}

		let wine_box = Self {
			folder,
			settings: Settings::new(),
		};
		let prefix = wine_box.prefix();
		let devices = prefix.join("dosdevices");

		create_folder(&devices)?;
		create_folder(&prefix.join("drive_c"))?;
		unix::fs::symlink("../drive_c", devices.join("c:")).map_err(|error| error.to_string())?;
		fs::write(
			prefix.join("user.reg"),
			include_str!(resource!("Registry/user.reg")),
		)
		.map_err(|error| error.to_string())?;
		wine_box.save()?;

		Ok(wine_box)
	}
}
