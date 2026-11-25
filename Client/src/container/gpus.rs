use std::{
	collections::BTreeMap,
	fs::{self, DirEntry},
	path::Path,
};

use super::host;

const PCI_IDS: &str = env!("EMPEROR_PENGUIN_PCI_IDS");
const DRM: &str = "/sys/class/drm";

#[derive(Clone, Copy)]
enum Driver {
	Nvidia,
	Nouveau,
	Amd,
	Intel,
}

impl Driver {
	fn from_name(name: &str) -> Option<Self> {
		match name {
			"nvidia" => Some(Self::Nvidia),
			"nouveau" => Some(Self::Nouveau),
			"amdgpu" | "radeon" => Some(Self::Amd),
			"i915" | "xe" => Some(Self::Intel),
			_ => None,
		}
	}

	const fn vendor(self) -> &'static str {
		match self {
			Self::Nvidia | Self::Nouveau => "NVIDIA",
			Self::Amd => "AMD",
			Self::Intel => "Intel",
		}
	}

	/// NVIDIA's comes with the host driver that the NVIDIA Container Toolkit mounts; Mesa's ship in
	/// the box.
	const fn vulkan_driver(self) -> &'static str {
		match self {
			Self::Nvidia => "/run/opengl-driver/share/vulkan/icd.d/nvidia_icd.json",
			Self::Nouveau => "/share/vulkan/icd.d/nouveau_icd.x86_64.json",
			Self::Amd => "/share/vulkan/icd.d/radeon_icd.x86_64.json",
			Self::Intel => "/share/vulkan/icd.d/intel_icd.x86_64.json",
		}
	}
}

/// Draws on the CPU, for a box that sees no GPU.
pub const SOFTWARE_VULKAN_DRIVER: &str = "/share/vulkan/icd.d/lvp_icd.x86_64.json";

pub struct Gpu {
	/// The PCI address, which stays the same across restarts.
	pub address: String,
	pub name: String,
	pub vulkan_driver: &'static str,
	/// Under `/dev/dri`.
	pub nodes: Vec<String>,
	/// The NVIDIA Container Toolkit's name for an NVIDIA GPU.
	pub toolkit_name: Option<String>,
}

fn numbered(name: &str, kind: &str) -> bool {
	name.strip_prefix(kind)
		.is_some_and(|number| number.parse::<u32>().is_ok())
}

/// A device node of a GPU in `/sys/class/drm`, with its GPU's PCI address.
fn node(entry: &DirEntry) -> Option<(String, String)> {
	let name = entry.file_name().into_string().ok()?;

	if !numbered(&name, "card") && !numbered(&name, "renderD") {
		return None;
	}

	let device = fs::canonicalize(entry.path().join("device")).ok()?;
	let address = device.file_name()?.to_str()?.to_owned();

	Some((name, address))
}

/// The NVIDIA Container Toolkit names GPUs by their device minor number. Without the toolkit a box has
/// no NVIDIA driver, so it can't draw on an NVIDIA GPU.
fn toolkit_name(address: &str) -> Option<String> {
	if !host::has_nvidia_toolkit() {
		return None;
	}

	let information =
		fs::read_to_string(format!("/proc/driver/nvidia/gpus/{address}/information")).ok()?;
	let minor = information
		.lines()
		.find_map(|line| line.strip_prefix("Device Minor:"))?;

	Some(minor.trim().to_owned())
}

fn read_id(device: &Path, file: &str) -> Option<String> {
	let text = fs::read_to_string(device.join(file)).ok()?;

	Some(text.trim().trim_start_matches("0x").to_owned())
}

fn is_device_line(line: &str) -> bool {
	line.starts_with('\t') || line.starts_with('#')
}

/// `pci.ids` lists each vendor's devices, indented, under the vendor's line. A bracketed name is
/// the product's, and the rest is its chip's.
fn product_name(ids: &str, vendor: &str, device: &str) -> Option<String> {
	let (vendor_line, device_line) = (format!("{vendor} "), format!("\t{device}  "));
	let mut lines = ids
		.lines()
		.skip_while(|line| !line.starts_with(&vendor_line));

	lines.next()?;

	let line = lines
		.take_while(|line| is_device_line(line))
		.find_map(|line| line.strip_prefix(&device_line))?;
	let product = line
		.split_once('[')
		.and_then(|(_, rest)| rest.strip_suffix(']'));

	Some(product.unwrap_or(line).to_owned())
}

impl Gpu {
	fn read(address: String, nodes: Vec<String>, ids: &str) -> Option<Self> {
		let device = Path::new(DRM).join(&nodes[0]).join("device");
		let driver_link = fs::read_link(device.join("driver")).ok()?;
		let driver = Driver::from_name(driver_link.file_name()?.to_str()?)?;
		let (vendor_id, device_id) = (read_id(&device, "vendor")?, read_id(&device, "device")?);
		let product = product_name(ids, &vendor_id, &device_id).unwrap_or(device_id);
		let toolkit_name = match driver {
			Driver::Nvidia => Some(toolkit_name(&address)?),
			Driver::Nouveau | Driver::Amd | Driver::Intel => None,
		};

		Some(Self {
			name: format!("{} {product}", driver.vendor()),
			address,
			vulkan_driver: driver.vulkan_driver(),
			nodes,
			toolkit_name,
		})
	}
}

/// The host's GPUs that a box can draw on, by PCI address.
pub fn list() -> Vec<Gpu> {
	let mut nodes: BTreeMap<String, Vec<String>> = BTreeMap::new();

	for (name, address) in fs::read_dir(DRM)
		.into_iter()
		.flatten()
		.flatten()
		.filter_map(|entry| node(&entry))
	{
		nodes.entry(address).or_default().push(name);
	}

	let ids = fs::read_to_string(PCI_IDS).expect("the panel's PCI ID list is readable");

	nodes
		.into_iter()
		.filter_map(|(address, nodes)| Gpu::read(address, nodes, &ids))
		.collect()
}
