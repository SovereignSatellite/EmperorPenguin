use std::{
	ffi::OsStr,
	path::{Path, PathBuf},
};

/// Holds no comma or line break, which would split Podman's `--mount` options and the lines of the box
/// list and settings files.
#[derive(Clone, PartialEq, Eq)]
pub struct Folder(String);

impl Folder {
	pub fn new(path: &Path) -> Result<Self, String> {
		let normalized: PathBuf = path.components().collect();
		let text = normalized
			.to_str()
			.ok_or("Folder paths must be valid Unicode.")?;

		if !normalized.is_absolute() || normalized.file_name().is_none() {
			return Err(format!("{text} is not a full path to a folder."));
		}
		if text.contains([',', '\n']) {
			return Err("Folder paths cannot contain commas or line breaks.".into());
		}

		Ok(Self(text.to_owned()))
	}

	pub fn path(&self) -> &Path {
		Path::new(&self.0)
	}

	pub fn text(&self) -> &str {
		&self.0
	}

	pub fn name(&self) -> &str {
		self.path()
			.file_name()
			.and_then(OsStr::to_str)
			.expect("a checked folder path ends in its name")
	}
}
