use core::iter;

use std::{
	env,
	ffi::OsString,
	fs, io,
	path::{Path, PathBuf},
};

fn is_set(value: &OsString) -> bool {
	!value.is_empty()
}

/// The XDG specification has programs ignore an empty or relative folder.
fn folder_variable(variable: &str) -> Option<PathBuf> {
	env::var_os(variable)
		.map(PathBuf::from)
		.filter(|folder| folder.is_absolute())
}

fn home_folder(variable: &str, fallback: &str) -> Result<PathBuf, String> {
	match folder_variable(variable) {
		Some(folder) => Ok(folder),
		None => Ok(PathBuf::from(env::var_os("HOME").ok_or("HOME is not set.")?).join(fallback)),
	}
}

pub fn configuration() -> Result<PathBuf, String> {
	Ok(home_folder("XDG_CONFIG_HOME", ".config")?.join("emperor-penguin"))
}

pub fn applications() -> Result<PathBuf, String> {
	Ok(home_folder("XDG_DATA_HOME", ".local/share")?.join("applications"))
}

pub fn all_applications() -> Result<Vec<PathBuf>, String> {
	let shared = env::var_os("XDG_DATA_DIRS")
		.filter(is_set)
		.unwrap_or_else(|| "/usr/local/share:/usr/share".into());
	let shared_applications = env::split_paths(&shared)
		.filter(|folder| folder.is_absolute())
		.map(|folder| folder.join("applications"));

	Ok(iter::once(applications()?)
		.chain(shared_applications)
		.collect())
}

pub fn runtime() -> Result<PathBuf, String> {
	folder_variable("XDG_RUNTIME_DIR").ok_or_else(|| "XDG_RUNTIME_DIR is not set.".into())
}

pub fn remove_if_present(path: &Path) -> io::Result<()> {
	match fs::remove_file(path) {
		Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
		result => result,
	}
}
