use std::{
	ffi::OsString,
	os::fd::{AsRawFd as _, OwnedFd},
	path::Path,
};

use super::{
	gpus::{self, Gpu},
	host,
};

use crate::{
	boxes::{Access, Drive, Settings, WineBox},
	links,
};

const ROOT: &str = env!("EMPEROR_PENGUIN_ROOT");
const STORE_PATHS: &str = include_str!(env!("EMPEROR_PENGUIN_STORE_PATHS"));

const OPTIONS: [&str; 16] = [
	"run",
	"--detach",
	"--rm",
	// Wine programs whose parent exited pass to the first process, which has to reap them.
	"--init",
	"--cap-drop=all",
	"--security-opt=no-new-privileges",
	"--read-only",
	"--read-only-tmpfs=false",
	concat!("--env-file=", env!("EMPEROR_PENGUIN_ENVIRONMENT")),
	// Podman and the NVIDIA hooks write mount points and files into these, and Podman gives them the
	// read-only modes of the root's folders unless told otherwise.
	"--mount=type=tmpfs,destination=/etc,tmpfs-mode=0755",
	"--mount=type=tmpfs,destination=/nix/store",
	"--mount=type=tmpfs,destination=/run,tmpfs-mode=0755",
	"--mount=type=tmpfs,destination=/tmp,tmpfs-mode=1777",
	"--mount=type=tmpfs,destination=/usr,tmpfs-mode=0755",
	"--mount=type=tmpfs,destination=/var,tmpfs-mode=0755",
	// Without a Z: drive, Wine cannot map the default working folder and warns about it.
	"--workdir=/prefix/drive_c",
];

fn bind(source: &Path, destination: &str, access: Access) -> OsString {
	let mut mount = OsString::from("--mount=type=bind,source=");

	mount.push(source);
	mount.push(",destination=");
	mount.push(destination);
	mount.push(match access {
		Access::ReadOnly => ",readonly",
		Access::ReadWrite => "",
	});

	mount
}

fn store_mount(path: &str) -> OsString {
	bind(Path::new(path), path, Access::ReadOnly)
}

pub const PREFIX: &str = "/prefix";

pub fn drive_folder(letter: char) -> String {
	format!("{PREFIX}/dosdevices/{letter}:")
}

fn drive_mount(drive: &Drive) -> OsString {
	let destination = drive_folder(drive.letter);

	bind(drive.folder.path(), &destination, drive.access)
}

fn box_mounts(wine_box: &WineBox, wayland: &Path) -> Result<Vec<OsString>, String> {
	let mut mounts = vec![
		bind(&wine_box.prefix(), PREFIX, Access::ReadWrite),
		bind(wayland, "/run/wayland", Access::ReadWrite),
		bind(&links::host_folder()?, links::BOX_FOLDER, Access::ReadOnly),
		// Controllers plugged in later appear in the bound folder, and host permissions keep keyboards
		// and mice out of reach.
		bind(Path::new("/dev/input"), "/dev/input", Access::ReadWrite),
	];

	mounts.extend(wine_box.settings.drives.iter().map(drive_mount));

	Ok(mounts)
}

fn setting_arguments(settings: &Settings, sound: Option<&Path>) -> Vec<OsString> {
	let mut arguments = Vec::new();

	if !settings.network {
		arguments.push("--network=none".into());
	}
	if let Some(sound) = sound {
		arguments.push(bind(sound, "/run/pulse/native", Access::ReadWrite));
	}
	if let Some(frames) = settings.frame_limit {
		let dxvk = format!("dxgi.maxFrameRate = {frames}; d3d9.maxFrameRate = {frames}");

		arguments.push(format!("--env=DXVK_CONFIG={dxvk}").into());
		arguments.push(format!("--env=VKD3D_FRAME_RATE={frames}").into());
	}

	arguments
}

fn device(path: &str) -> OsString {
	format!("--device={path}").into()
}

fn dri_device(node: &String) -> OsString {
	device(&format!("/dev/dri/{node}"))
}

fn toolkit_device(gpu: &Gpu) -> Option<OsString> {
	let name = gpu.toolkit_name.as_ref()?;

	Some(device(&format!("nvidia.com/gpu={name}")))
}

/// Each visible GPU brings its device nodes and one Vulkan driver, so programs see each GPU once.
fn gpu_arguments(settings: &Settings) -> Vec<OsString> {
	let gpus: Vec<Gpu> = gpus::list()
		.into_iter()
		.filter(|gpu| !settings.hidden_gpus.contains(&gpu.address))
		.collect();
	let mut arguments: Vec<OsString> = gpus
		.iter()
		.flat_map(|gpu| &gpu.nodes)
		.map(dri_device)
		.collect();

	arguments.extend(gpus.iter().filter_map(toolkit_device));

	let mut drivers: Vec<&str> = gpus.iter().map(|gpu| gpu.vulkan_driver).collect();

	drivers.sort_unstable();
	drivers.dedup();

	if drivers.is_empty() {
		drivers.push(gpus::SOFTWARE_VULKAN_DRIVER);
	}

	arguments.push(format!("--env=VK_DRIVER_FILES={}", drivers.join(":")).into());

	arguments
}

fn host_devices() -> Vec<OsString> {
	let mut devices = Vec::new();

	if host::has_ntsync() {
		devices.push("--device=/dev/ntsync".into());
	}

	devices
}

fn preserved(file: &OwnedFd) -> OsString {
	format!("--preserve-fd={}", file.as_raw_fd()).into()
}

/// Wine's server ends the box a few seconds after the last program exits, once it has closed
/// Wine's own background programs.
const COMMAND: [&str; 5] = [
	"--rootfs",
	ROOT,
	"/bin/wineserver",
	"--foreground",
	"--persistent=5",
];

/// The box holds each `held` pipe end open while it runs, which keeps its Wayland and sound sockets
/// served.
pub fn session(
	wine_box: &WineBox,
	name: &str,
	wayland: &Path,
	sound: Option<&Path>,
	held: &[OwnedFd],
) -> Result<Vec<OsString>, String> {
	let mut arguments: Vec<OsString> = OPTIONS.map(OsString::from).to_vec();

	arguments.push(format!("--name={name}").into());
	arguments.push(
		format!(
			"--env=WINE_WAYLAND_APP_ID_PREFIX={}.",
			wine_box.folder.name()
		)
		.into(),
	);
	arguments.extend(held.iter().map(preserved));
	arguments.extend(STORE_PATHS.lines().map(store_mount));
	arguments.extend(box_mounts(wine_box, wayland)?);
	arguments.extend(setting_arguments(&wine_box.settings, sound));
	arguments.extend(gpu_arguments(&wine_box.settings));
	arguments.extend(host_devices());
	arguments.extend(COMMAND.map(OsString::from));

	Ok(arguments)
}
