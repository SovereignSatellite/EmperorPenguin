use core::num::NonZeroU32;

use gpui::IntoElement;
use gpui_component::{
	Disableable as _,
	button::Button,
	menu::{DropdownMenu as _, PopupMenu, PopupMenuItem},
};

use super::icons;

use crate::interface::panel::{BoxHandle, Entry};

const fn percent(value: u32) -> NonZeroU32 {
	NonZeroU32::new(value).unwrap()
}

const SCALES: [NonZeroU32; 7] = [
	percent(100),
	percent(125),
	percent(150),
	percent(175),
	percent(200),
	percent(250),
	percent(300),
];
const FRAME_LIMITS: [Option<NonZeroU32>; 8] = [
	None,
	NonZeroU32::new(30),
	NonZeroU32::new(60),
	NonZeroU32::new(90),
	NonZeroU32::new(120),
	NonZeroU32::new(144),
	NonZeroU32::new(165),
	NonZeroU32::new(240),
];

fn frame_limit_text(limit: Option<NonZeroU32>) -> String {
	limit.map_or_else(
		|| "No FPS limit".to_owned(),
		|frames| format!("{frames} FPS"),
	)
}

fn scale_item(handle: &BoxHandle, scale: NonZeroU32, current: NonZeroU32) -> PopupMenuItem {
	let handle = handle.clone();

	PopupMenuItem::new(format!("{scale}%"))
		.checked(scale == current)
		.on_click(move |_, _, context| handle.set_scale(scale, context))
}

fn frame_limit_item(
	handle: &BoxHandle,
	limit: Option<NonZeroU32>,
	current: Option<NonZeroU32>,
) -> PopupMenuItem {
	let handle = handle.clone();

	PopupMenuItem::new(frame_limit_text(limit))
		.checked(limit == current)
		.on_click(move |_, _, context| handle.set_frame_limit(limit, context))
}

fn scale_menu(mut menu: PopupMenu, handle: &BoxHandle, current: NonZeroU32) -> PopupMenu {
	for scale in SCALES {
		menu = menu.item(scale_item(handle, scale, current));
	}

	menu
}

fn frame_limit_menu(
	mut menu: PopupMenu,
	handle: &BoxHandle,
	current: Option<NonZeroU32>,
) -> PopupMenu {
	for limit in FRAME_LIMITS {
		menu = menu.item(frame_limit_item(handle, limit, current));
	}

	menu
}

pub fn scale_button(handle: &BoxHandle, entry: &Entry) -> impl IntoElement {
	let (handle, current) = (handle.clone(), entry.wine_box.settings.scale);

	Button::new("scale")
		.outline()
		.icon(icons::scale())
		.label(format!("{current}%"))
		.tooltip("Scale")
		.disabled(!entry.editable())
		.dropdown_menu(move |menu, _, _| scale_menu(menu, &handle, current))
}

pub fn frame_limit_button(handle: &BoxHandle, entry: &Entry) -> impl IntoElement {
	let (handle, current) = (handle.clone(), entry.wine_box.settings.frame_limit);

	Button::new("frame-limit")
		.outline()
		.icon(icons::frame_rate())
		.label(frame_limit_text(current))
		.tooltip("Frame rate limit for DirectX games")
		.disabled(!entry.editable())
		.dropdown_menu(move |menu, _, _| frame_limit_menu(menu, &handle, current))
}
