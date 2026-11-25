use std::{fs, io};

use super::Appearance;

use crate::folders;

const FILE_NAME: &str = "theme";

impl Appearance {
	fn parse(text: &str) -> Option<Self> {
		match text {
			"light\n" => Some(Self::Light),
			"dark\n" => Some(Self::Dark),
			"auto\n" => Some(Self::Automatic),
			_ => None,
		}
	}

	const fn serialize(self) -> &'static str {
		match self {
			Self::Light => "light\n",
			Self::Dark => "dark\n",
			Self::Automatic => "auto\n",
		}
	}
}

pub fn read() -> Result<Appearance, String> {
	let file = folders::configuration()?.join(FILE_NAME);

	match fs::read_to_string(&file) {
		Ok(text) => Appearance::parse(&text)
			.ok_or_else(|| format!("{} holds no theme mode.", file.display())),
		Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Appearance::Automatic),
		Err(error) => Err(format!("{}: {error}", file.display())),
	}
}

pub fn write(appearance: Appearance) -> Result<(), String> {
	let folder = folders::configuration()?;

	fs::create_dir_all(&folder).map_err(|error| error.to_string())?;

	fs::write(folder.join(FILE_NAME), appearance.serialize()).map_err(|error| error.to_string())
}
