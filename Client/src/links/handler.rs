use std::{
	collections::BTreeSet,
	fs,
	path::{Path, PathBuf},
	process::Command,
};

use super::{LinkKind, registry};

use crate::folders;

const UPDATE_DESKTOP_DATABASE: &str = env!("EMPEROR_PENGUIN_UPDATE_DESKTOP_DATABASE");
const ENTRY: &str = "dev.sovereignsatellite.EmperorPenguin.Links.desktop";
const SCHEME_HANDLER: &str = "x-scheme-handler/";

fn entry<'kind, I>(kinds: I) -> String
where
	I: IntoIterator<Item = &'kind LinkKind>,
{
	let mut entry = String::from(
		"[Desktop Entry]\nType=Application\nName=Emperor Penguin\nNoDisplay=true\nExec=emperor-penguin %u\nMimeType=",
	);

	for kind in kinds {
		entry.push_str("x-scheme-handler/");
		entry.push_str(kind.as_str());
		entry.push(';');
	}

	entry.push('\n');

	entry
}

fn names_another_program(entry: &str) -> bool {
	!entry.is_empty() && entry != ENTRY
}

fn add_host_kinds(cache: &str, kinds: &mut BTreeSet<LinkKind>) {
	for line in cache.lines() {
		if let Some((kind, entries)) = line
			.strip_prefix(SCHEME_HANDLER)
			.and_then(|handler| handler.split_once('='))
			&& entries.split(';').any(names_another_program)
			&& let Some(kind) = LinkKind::new(kind)
		{
			kinds.insert(kind);
		}
	}
}

fn host_kinds() -> Result<BTreeSet<LinkKind>, String> {
	let mut kinds = BTreeSet::new();

	for folder in folders::all_applications()? {
		if let Ok(cache) = fs::read_to_string(folder.join("mimeinfo.cache")) {
			add_host_kinds(&cache, &mut kinds);
		}
	}

	Ok(kinds)
}

/// `xdg-open` looks a link kind up in this cache, and finds the user's before the system's.
fn update_database(folder: &Path) -> Result<(), String> {
	let output = Command::new(UPDATE_DESKTOP_DATABASE)
		.arg(folder)
		.output()
		.map_err(|error| error.to_string())?;

	if !output.status.success() {
		return Err(format!(
			"update-desktop-database failed with {}: {}",
			output.status,
			String::from_utf8_lossy(&output.stderr).trim()
		));
	}

	Ok(())
}

/// The desktop reads the panel's entry before the system's, so a box that registered a kind a host program
/// handles would receive that program's links, such as its sign-in replies.
pub fn register(prefixes: &[PathBuf]) -> Result<(), String> {
	let host = host_kinds()?;
	let boxes: BTreeSet<LinkKind> = prefixes
		.iter()
		.flat_map(|prefix| registry::box_kinds(prefix))
		.collect();
	let folder = folders::applications()?;

	fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
	fs::write(folder.join(ENTRY), entry(boxes.difference(&host)))
		.map_err(|error| error.to_string())?;

	update_database(&folder)
}
