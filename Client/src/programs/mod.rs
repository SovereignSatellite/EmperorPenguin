mod bytes;
mod icon;
mod shortcut;

use core::iter;

use std::{
	collections::HashSet,
	fs,
	path::{Path, PathBuf},
};

use shortcut::Shortcut;

use crate::boxes::WineBox;

const LONGEST_SHORTCUT: u64 = 64 << 10;
const DEEPEST_FOLDER: usize = 8;
const MOST_SHORTCUTS: usize = 1000;
const START_MENU: [&str; 4] = ["Microsoft", "Windows", "Start Menu", "Programs"];

pub struct Program {
	pub name: String,
	pub icon: Option<Vec<u8>>,
	pub shortcut: PathBuf,
}

/// Windows names files case-insensitively. Refuses symbolic links, since programs in the box can plant
/// them in its folders.
fn child(folder: &Path, name: &str) -> Option<PathBuf> {
	fs::read_dir(folder)
		.ok()?
		.flatten()
		.find(|entry| {
			entry
				.file_name()
				.to_str()
				.is_some_and(|entry_name| entry_name.eq_ignore_ascii_case(name))
		})
		.filter(|entry| entry.file_type().is_ok_and(|kind| !kind.is_symlink()))
		.map(|entry| entry.path())
}

fn resolve<'name, I>(root: PathBuf, names: I) -> Option<PathBuf>
where
	I: IntoIterator<Item = &'name str>,
{
	names
		.into_iter()
		.try_fold(root, |folder, name| child(&folder, name))
}

fn host_path(wine_box: &WineBox, windows_path: &str) -> Option<PathBuf> {
	let (drive, rest) = windows_path.split_once(":\\")?;
	let &[letter] = drive.as_bytes() else {
		return None;
	};
	let letter = char::from(letter.to_ascii_lowercase());
	let names = rest.split('\\');

	if letter == 'c' {
		return resolve(wine_box.prefix(), iter::once("drive_c").chain(names));
	}

	let mounted = wine_box
		.settings
		.drives
		.iter()
		.find(|mounted| mounted.letter == letter)?;

	resolve(mounted.folder.path().to_path_buf(), names)
}

/// Programs in the box can make a folder or entry the panel can't read, which then holds no shortcuts.
fn add_shortcuts(folder: &Path, depth: usize, found: &mut Vec<PathBuf>) {
	let Ok(entries) = fs::read_dir(folder) else {
		return;
	};

	for entry in entries.flatten() {
		if found.len() == MOST_SHORTCUTS {
			break;
		}

		let Ok(kind) = entry.file_type() else {
			continue;
		};
		let path = entry.path();

		if kind.is_dir() && depth > 0 {
			add_shortcuts(&path, depth - 1, found);
		} else if path
			.extension()
			.is_some_and(|extension| extension.eq_ignore_ascii_case("lnk"))
		{
			found.push(path);
		}
	}
}

fn add_user_shortcuts(home: PathBuf, found: &mut Vec<PathBuf>) {
	if let Some(desktop) = child(&home, "Desktop") {
		add_shortcuts(&desktop, 0, found);
	}

	if let Some(menu) = resolve(home, ["AppData", "Roaming"].into_iter().chain(START_MENU)) {
		add_shortcuts(&menu, DEEPEST_FOLDER, found);
	}
}

fn shortcut_files(prefix: PathBuf) -> Vec<PathBuf> {
	let mut found = Vec::new();
	let shared_menu = ["drive_c", "ProgramData"].into_iter().chain(START_MENU);

	if let Some(menu) = resolve(prefix.clone(), shared_menu) {
		add_shortcuts(&menu, DEEPEST_FOLDER, &mut found);
	}

	let Some(users) = resolve(prefix, ["drive_c", "users"]) else {
		return found;
	};

	for user in fs::read_dir(users).into_iter().flatten().flatten() {
		if user.file_type().is_ok_and(|kind| kind.is_dir()) {
			add_user_shortcuts(user.path(), &mut found);
		}
	}

	found
}

fn program_shortcut(path: &Path) -> Option<(String, Shortcut, PathBuf)> {
	let name = path.file_stem()?.to_str()?.to_owned();

	if name.to_lowercase().starts_with("uninstall") {
		return None;
	}

	let shortcut = Shortcut::parse(&bytes::read_file(path, LONGEST_SHORTCUT)?)?;

	shortcut
		.target
		.to_lowercase()
		.ends_with(".exe")
		.then_some((name, shortcut, path.to_owned()))
}

impl Program {
	fn from_shortcut(wine_box: &WineBox, name: String, shortcut: &Shortcut, file: PathBuf) -> Self {
		let location = shortcut
			.icon_location
			.as_deref()
			.unwrap_or(&shortcut.target);
		let icon = host_path(wine_box, location)
			.and_then(|path| icon::extract(&path, shortcut.icon_index));

		Self {
			name,
			icon,
			shortcut: file,
		}
	}
}

pub fn installed(wine_box: &WineBox) -> Vec<Program> {
	let files = shortcut_files(wine_box.prefix());
	let mut shortcuts: Vec<_> = files
		.iter()
		.filter_map(|path| program_shortcut(path))
		.collect();
	let mut targets = HashSet::new();

	shortcuts.sort_by_cached_key(|(name, _, _)| name.to_lowercase());
	shortcuts.retain(|(_, shortcut, _)| targets.insert(shortcut.target.to_lowercase()));

	shortcuts
		.into_iter()
		.map(|(name, shortcut, file)| Program::from_shortcut(wine_box, name, &shortcut, file))
		.collect()
}
