mod arguments;
mod gpus;
mod host;
mod mount_points;
mod podman;
mod registry;
mod translation;
mod wayland;
mod wine_build;

use std::{
	ffi::OsString,
	fs,
	path::{Path, PathBuf},
};

pub use gpus::{Gpu, list as gpus};
pub use registry::Shade;

use crate::{
	boxes::{self, Drive, Settings, WineBox},
	folders,
	sound::{self, Grant},
};

pub enum Target {
	File(PathBuf),
	Program(&'static str),
}

fn fnv_step(hash: u64, byte: u8) -> u64 {
	(hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
}

/// FNV-1a stays stable across builds, unlike the standard library's hashers, so a new build still
/// finds the containers that an earlier one started.
fn name(wine_box: &WineBox) -> String {
	let hash = wine_box
		.folder
		.text()
		.bytes()
		.fold(0xcbf2_9ce4_8422_2325, fnv_step);

	format!("emperor-penguin-{hash:016x}")
}

/// Each box gets its own sockets, outside the folder that boxes share.
fn socket(kind: &str, name: &str) -> Result<PathBuf, String> {
	let folder = folders::runtime()?.join(kind);

	fs::create_dir_all(&folder).map_err(|error| error.to_string())?;

	Ok(folder.join(name))
}

const fn grant(settings: &Settings) -> Option<Grant> {
	if !settings.sound && !settings.microphone {
		return None;
	}

	Some(Grant {
		playback: settings.sound,
		recording: settings.microphone,
	})
}

pub fn start(wine_box: &WineBox, shade: Shade) -> Result<(), String> {
	wine_box
		.settings
		.drives
		.iter()
		.try_for_each(boxes::check_drive)?;

	let name = name(wine_box);
	let wayland = socket("emperor-penguin-wayland", &name)?;

	mount_points::remove_unused(wine_box);
	translation::install(wine_box)?;
	wine_build::refresh_prefix(wine_box)?;
	registry::apply(wine_box, shade).map_err(|error| error.to_string())?;

	let mut held = vec![wayland::filtered_socket(&wayland, &name)?];
	let sound = match grant(&wine_box.settings) {
		Some(grant) => {
			let sound = socket("emperor-penguin-sound", &name)?;

			held.push(sound::start(&sound, grant)?);

			Some(sound)
		}
		None => None,
	};
	let arguments = arguments::session(wine_box, &name, &wayland, sound.as_deref(), &held)?;

	podman::run_holding(arguments, held).map(drop)
}

fn drive_path(drive: &Drive, file: &Path) -> Option<PathBuf> {
	let inside = file.strip_prefix(drive.folder.path()).ok()?;

	Some(Path::new(&arguments::drive_folder(drive.letter)).join(inside))
}

fn mapped(wine_box: &WineBox, file: &Path) -> Option<PathBuf> {
	if let Ok(inside) = file.strip_prefix(wine_box.prefix()) {
		return Some(Path::new(arguments::PREFIX).join(inside));
	}

	wine_box
		.settings
		.drives
		.iter()
		.find_map(|drive| drive_path(drive, file))
}

fn reachable(wine_box: &WineBox, file: &Path) -> Result<PathBuf, String> {
	match mapped(wine_box, file) {
		Some(path) => Ok(path),
		None => Ok(Path::new(arguments::PREFIX).join(boxes::import(wine_box, file)?)),
	}
}

/// A program started from outside the box waits until the box's Wine session has started, which
/// includes setting up a new prefix.
pub fn launch(wine_box: &WineBox, target: &Target) -> Result<(), String> {
	let mut command = ["exec", &name(wine_box), "wine", "start"]
		.map(OsString::from)
		.to_vec();

	match target {
		Target::File(file) => command.extend(["/unix".into(), reachable(wine_box, file)?.into()]),
		Target::Program(program) => command.push(program.into()),
	}

	podman::run(command).map(drop)
}

pub fn stop(wine_box: &WineBox) {
	let name = name(wine_box);

	podman::attempt([
		"exec",
		&name,
		"wine",
		"wineboot",
		"--end-session",
		"--shutdown",
	]);
	podman::attempt(["exec", &name, "wineserver", "--kill"]);
}

pub struct Running {
	names: String,
}

impl Running {
	pub fn query() -> Result<Self, String> {
		podman::run(["ps", "--format={{.Names}}"]).map(|names| Self { names })
	}

	pub fn contains(&self, wine_box: &WineBox) -> bool {
		let name = name(wine_box);

		self.names.lines().any(|running| running == name)
	}
}

pub fn wait(wine_box: &WineBox) {
	podman::attempt(["wait", &name(wine_box)]);
}

/// Wine's `start` overflows a fixed buffer on links over 259 characters, so the link goes straight
/// to the shell.
pub fn open(wine_box: &WineBox, link: &str) -> Result<(), String> {
	podman::run([
		"exec",
		&name(wine_box),
		"wine",
		"rundll32",
		"shell32.dll,ShellExec_RunDLL",
		link,
	])
	.map(drop)
}
