use std::{
	ffi::OsStr,
	fs::{self, File, OpenOptions},
	io,
	os::{fd::OwnedFd, unix::fs::OpenOptionsExt as _},
	path::{Path, PathBuf},
};

use rustix::{
	fs::{AtFlags, Mode, OFlags, mkdirat, open, openat, unlinkat},
	io::Errno,
};

use crate::folders;

/// The x86-64 Linux values, the only target the flake builds for.
const O_NONBLOCK: i32 = 0o4000;
const O_NOFOLLOW: i32 = 0o400_000;

// Programs in a box can replace its files and folders with symbolic links out of it, or with FIFOs
// that would block a read.

pub fn open_regular(path: &Path) -> io::Result<File> {
	let file = OpenOptions::new()
		.read(true)
		.custom_flags(O_NONBLOCK | O_NOFOLLOW)
		.open(path)?;

	if !file.metadata()?.is_file() {
		return Err(io::Error::new(
			io::ErrorKind::InvalidData,
			format!("{} is not a regular file", path.display()),
		));
	}

	Ok(file)
}

pub fn create_regular(path: &Path) -> io::Result<File> {
	folders::remove_if_present(path)?;

	OpenOptions::new().write(true).create_new(true).open(path)
}

pub fn real_folder<'name, I>(root: &Path, names: I) -> Result<PathBuf, String>
where
	I: IntoIterator<Item = &'name str>,
{
	let mut folder = root.to_path_buf();

	for name in names {
		folder.push(name);

		match fs::symlink_metadata(&folder) {
			Ok(metadata) if metadata.is_dir() => {}
			Ok(_) => return Err(format!("{} is not a folder.", folder.display())),
			Err(missing) if missing.kind() == io::ErrorKind::NotFound => {
				fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
			}
			Err(error) => return Err(error.to_string()),
		}
	}

	Ok(folder)
}

fn open_folder(parent: &OwnedFd, name: &str) -> Result<OwnedFd, Errno> {
	openat(
		parent,
		name,
		OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
		Mode::empty(),
	)
}

fn open_or_make_folder(parent: &OwnedFd, name: &str) -> Result<OwnedFd, Errno> {
	match open_folder(parent, name) {
		Err(Errno::NOENT) => {
			mkdirat(parent, name, Mode::from_bits_truncate(0o777))?;

			open_folder(parent, name)
		}
		opened => opened,
	}
}

/// Holds each folder open on the way, since a program in a running box can swap one for a symbolic
/// link out of the box between a check and a use of its path.
pub fn create_inside<'name, I>(root: &Path, names: I, file: &OsStr) -> io::Result<File>
where
	I: IntoIterator<Item = &'name str>,
{
	let mut folder = open(root, OFlags::DIRECTORY | OFlags::CLOEXEC, Mode::empty())?;

	for name in names {
		folder = open_or_make_folder(&folder, name)?;
	}

	match unlinkat(&folder, file, AtFlags::empty()) {
		Ok(()) | Err(Errno::NOENT) => {}
		Err(error) => return Err(error.into()),
	}

	let created = openat(
		&folder,
		file,
		OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
		Mode::from_bits_truncate(0o666),
	)?;

	Ok(created.into())
}
