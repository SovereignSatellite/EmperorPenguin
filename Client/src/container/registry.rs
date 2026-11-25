use core::iter;

use std::{
	fs,
	io::{self, Read as _, Write as _},
};

use crate::boxes::{WineBox, create_regular, open_regular};

/// Programs in the box can make the registry file any size.
const LARGEST_REGISTRY: u64 = 64 << 20;

/// Each value is Windows' `AppsUseLightTheme` and the index of its color in the palette.
#[derive(Clone, Copy)]
pub enum Shade {
	Dark = 0,
	Light = 1,
}

struct Value {
	section: &'static str,
	/// Quoted, with its `=`.
	name: &'static str,
	data: String,
}

const DESKTOP: &str = r"[Control Panel\\Desktop]";
const METRICS: &str = r"[Control Panel\\Desktop\\WindowMetrics]";
const PERSONALIZE: &str = r"[Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize]";
const REPLACEMENTS: &str = r"[Software\\Wine\\Fonts\\Replacements]";
const COLORS: &str = r"[Control Panel\\Colors]";

const UI_FONT: &str = "Geist";
const UI_FONT_POINTS: u32 = 9;
const UI_FONTS: [(&str, u32); 6] = [
	("\"CaptionFont\"=", 500),
	("\"SmCaptionFont\"=", 500),
	("\"MenuFont\"=", 400),
	("\"MessageFont\"=", 400),
	("\"StatusFont\"=", 400),
	("\"IconFont\"=", 400),
];
/// Programs ask for these by name, and the box doesn't have them.
const SUBSTITUTES: [(&str, &str); 2] =
	[("\"Segoe UI\"=", "Geist"), ("\"Consolas\"=", "Geist Mono")];
const LIGHT_THEME: [&str; 2] = ["\"AppsUseLightTheme\"=", "\"SystemUsesLightTheme\"="];

/// The panel's colors, indexed by `Shade`. Classic 3D edges draw an outer and an inner line on each
/// side, so the outer lines take the border color and the inner ones the surface color or a faint
/// shadow, leaving flat outlines.
const PALETTE: [(&str, [&str; 2]); 30] = [
	("\"ActiveBorder\"=", ["10 10 10", "255 255 255"]),
	("\"ActiveTitle\"=", ["10 10 10", "255 255 255"]),
	("\"AppWorkSpace\"=", ["0 0 0", "250 250 250"]),
	("\"Background\"=", ["0 0 0", "250 250 250"]),
	("\"ButtonAlternateFace\"=", ["26 26 26", "242 242 242"]),
	("\"ButtonDkShadow\"=", ["46 46 46", "235 235 235"]),
	("\"ButtonFace\"=", ["10 10 10", "255 255 255"]),
	("\"ButtonHilight\"=", ["10 10 10", "255 255 255"]),
	("\"ButtonLight\"=", ["46 46 46", "235 235 235"]),
	("\"ButtonShadow\"=", ["31 31 31", "242 242 242"]),
	("\"ButtonText\"=", ["237 237 237", "23 23 23"]),
	("\"GradientActiveTitle\"=", ["10 10 10", "255 255 255"]),
	("\"GradientInactiveTitle\"=", ["10 10 10", "255 255 255"]),
	("\"GrayText\"=", ["102 102 102", "143 143 143"]),
	("\"Hilight\"=", ["51 51 51", "224 224 224"]),
	("\"HilightText\"=", ["237 237 237", "23 23 23"]),
	("\"HotTrackingColor\"=", ["82 168 255", "0 107 255"]),
	("\"InactiveBorder\"=", ["10 10 10", "255 255 255"]),
	("\"InactiveTitle\"=", ["10 10 10", "255 255 255"]),
	("\"InactiveTitleText\"=", ["160 160 160", "102 102 102"]),
	("\"InfoText\"=", ["237 237 237", "23 23 23"]),
	("\"InfoWindow\"=", ["10 10 10", "255 255 255"]),
	("\"Menu\"=", ["10 10 10", "255 255 255"]),
	("\"MenuBar\"=", ["10 10 10", "255 255 255"]),
	("\"MenuHilight\"=", ["31 31 31", "235 235 235"]),
	("\"MenuText\"=", ["237 237 237", "23 23 23"]),
	("\"Scrollbar\"=", ["0 0 0", "250 250 250"]),
	("\"TitleText\"=", ["237 237 237", "23 23 23"]),
	("\"Window\"=", ["0 0 0", "255 255 255"]),
	("\"WindowText\"=", ["237 237 237", "23 23 23"]),
];

/// A `LOGFONTW`. Its positive height counts points, which Wine scales to the box's resolution.
fn ui_font(weight: u32) -> String {
	let numbers = [UI_FONT_POINTS, 0, 0, 0, weight]
		.into_iter()
		.flat_map(u32::to_le_bytes);
	let flags = [0, 0, 0, 1, 0, 0, 0, 0];
	let face = UI_FONT.encode_utf16().chain(iter::repeat(0)).take(32);
	let bytes = numbers.chain(flags).chain(face.flat_map(u16::to_le_bytes));
	let hex: Vec<String> = bytes.map(|byte| format!("{byte:02x}")).collect();

	format!("hex:{}", hex.join(","))
}

fn values(wine_box: &WineBox, shade: Shade) -> Vec<Value> {
	let pixels_per_inch = wine_box.settings.scale.get() * 96 / 100;
	let fonts = UI_FONTS.map(|(name, weight)| Value {
		section: METRICS,
		name,
		data: ui_font(weight),
	});
	let substitutes = SUBSTITUTES.map(|(name, font)| Value {
		section: REPLACEMENTS,
		name,
		data: format!("\"{font}\""),
	});
	let theme = LIGHT_THEME.map(|name| Value {
		section: PERSONALIZE,
		name,
		data: format!("dword:{:08x}", shade as u32),
	});
	let colors = PALETTE.map(|(name, colors)| Value {
		section: COLORS,
		name,
		data: format!("\"{}\"", colors[shade as usize]),
	});
	let scale = Value {
		section: DESKTOP,
		name: "\"LogPixels\"=",
		data: format!("dword:{pixels_per_inch:08x}"),
	};

	iter::once(scale)
		.chain(fonts)
		.chain(substitutes)
		.chain(theme)
		.chain(colors)
		.collect()
}

fn is_section(header: &str, section: &str) -> bool {
	header
		.strip_prefix(section)
		.is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
}

fn value_at(values: &[Value], header: &str, line: &str) -> Option<usize> {
	values
		.iter()
		.position(|value| line.starts_with(value.name) && is_section(header, value.section))
}

/// Wine saves a key only when it changes, so appending values on each start would pile up copies.
/// Values the registry lacks go into repeated sections, which Wine's loader merges.
fn with_values(registry: &str, values: &[Value]) -> String {
	let mut updated = String::with_capacity(registry.len());
	let mut found = vec![false; values.len()];
	let mut header = "";
	let mut lines = registry.lines();

	while let Some(line) = lines.next() {
		if line.starts_with('[') {
			header = line;
		}

		match value_at(values, header, line) {
			Some(index) => {
				found[index] = true;

				if line.ends_with('\\') {
					lines.find(|continuation| !continuation.ends_with('\\'));
				}

				updated.extend([values[index].name, &values[index].data, "\n"]);
			}
			None => updated.extend([line, "\n"]),
		}
	}

	let mut appending = "";

	for (value, _) in values.iter().zip(found).filter(|(_, found)| !found) {
		if value.section != appending {
			appending = value.section;

			updated.extend(["\n", appending, " 0\n"]);
		}

		updated.extend([value.name, &value.data, "\n"]);
	}

	updated
}

/// Each Wine program reads these once as it starts, so they go into the registry before the box
/// starts.
pub fn apply(wine_box: &WineBox, shade: Shade) -> io::Result<()> {
	let prefix = wine_box.prefix();
	let path = prefix.join("user.reg");
	let mut registry = String::new();

	open_regular(&path)?
		.take(LARGEST_REGISTRY + 1)
		.read_to_string(&mut registry)?;

	if registry.len() as u64 > LARGEST_REGISTRY {
		return Err(io::Error::other(format!(
			"{} is over {} MiB, more than any real registry.",
			path.display(),
			LARGEST_REGISTRY >> 20
		)));
	}

	let updated = with_values(&registry, &values(wine_box, shade));

	if updated == registry {
		return Ok(());
	}

	let staging = prefix.join("user.reg.panel");

	create_regular(&staging)?.write_all(updated.as_bytes())?;

	fs::rename(staging, path)
}
