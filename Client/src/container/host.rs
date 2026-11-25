use std::fs::{self, DirEntry, File};

fn is_gpu_specification(entry: &DirEntry) -> bool {
	fs::read_to_string(entry.path()).is_ok_and(|text| text.contains("nvidia.com/gpu"))
}

pub fn has_nvidia_toolkit() -> bool {
	["/etc/cdi", "/var/run/cdi"]
		.into_iter()
		.flat_map(fs::read_dir)
		.flatten()
		.flatten()
		.any(|entry| is_gpu_specification(&entry))
}

/// Podman refuses to start a container with a device that the user cannot open, and Wine opens
/// this one read-only.
pub fn has_ntsync() -> bool {
	File::open("/dev/ntsync").is_ok()
}
