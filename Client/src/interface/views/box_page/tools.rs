use gpui::{App, IntoElement};
use gpui_component::{
	Disableable as _,
	button::Button,
	menu::{DropdownMenu as _, PopupMenu, PopupMenuItem},
};

use super::icons;

use crate::{container::Target, interface::panel::BoxHandle};

const TOOLS: [(&str, &str); 5] = [
	("Files", "explorer"),
	("Wine settings", "winecfg"),
	("Task Manager", "taskmgr"),
	("Command Prompt", "cmd"),
	("Registry Editor", "regedit"),
];

fn launch(handle: &BoxHandle, program: &'static str, context: &mut App) {
	handle.launch(Target::Program(program), context);
}

fn tools_menu(mut menu: PopupMenu, handle: &BoxHandle) -> PopupMenu {
	for (label, program) in TOOLS {
		let handle = handle.clone();

		menu = menu.item(
			PopupMenuItem::new(label)
				.on_click(move |_, _, context| launch(&handle, program, context)),
		);
	}

	menu
}

pub fn tools_button(handle: &BoxHandle, busy: bool) -> impl IntoElement {
	let handle = handle.clone();

	Button::new("tools")
		.outline()
		.icon(icons::tools())
		.tooltip("Tools")
		.disabled(busy)
		.dropdown_menu(move |menu, _, _| tools_menu(menu, &handle))
}
