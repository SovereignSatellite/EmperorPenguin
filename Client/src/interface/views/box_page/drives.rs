use gpui::{
	App, Div, InteractiveElement as _, ParentElement as _, Stateful,
	StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _,
};
use gpui_component::{
	ActiveTheme as _, Disableable as _, Icon, IconName,
	button::{Button, ButtonVariants as _},
};

use super::icons;

use crate::{
	boxes::{Access, Drive},
	interface::{
		panel::{BoxHandle, Entry, Usage},
		views::{
			card::{card, divided, leading, muted, row},
			path::path,
			removal::removal,
		},
	},
};

fn drive(letter: char, folder: &str, context: &App) -> Div {
	row()
		.child(leading(format!("{}:", letter.to_ascii_uppercase())))
		.child(path(folder, context))
}

fn highlighted(row: Div, context: &App) -> Div {
	let hover = context.theme().list_hover;

	row.hover(move |style| style.bg(hover))
}

fn access_button(handle: &BoxHandle, index: usize, access: Access, editable: bool) -> Button {
	let (other, tooltip) = match access {
		Access::ReadOnly => (Access::ReadWrite, "Read-only"),
		Access::ReadWrite => (Access::ReadOnly, "Read and write"),
	};
	let handle = handle.clone();

	Button::new(("access", index))
		.outline()
		.icon(icons::access(access))
		.tooltip(tooltip)
		.disabled(!editable)
		.on_click(move |_, _, context| handle.set_access(index, other, context))
}

fn remove_button(handle: &BoxHandle, index: usize, editable: bool, context: &App) -> Button {
	let handle = handle.clone();

	Button::new(("remove-drive", index))
		.custom(removal(context))
		.icon(IconName::Close)
		.tooltip("Remove drive")
		.disabled(!editable)
		.on_click(move |_, _, context| handle.remove_drive(index, context))
}

fn mounted(
	handle: &BoxHandle,
	index: usize,
	settings: &Drive,
	editable: bool,
	context: &App,
) -> Div {
	divided(
		drive(settings.letter, settings.folder.text(), context),
		context,
	)
	.when(editable, |row| highlighted(row, context))
	.child(access_button(handle, index, settings.access, editable))
	.child(remove_button(handle, index, editable, context))
}

fn add_row(handle: &BoxHandle, editable: bool, context: &App) -> Stateful<Div> {
	let row = divided(row(), context)
		.child(leading(Icon::new(IconName::Plus)))
		.child(div().text_sm().child("Add drive"))
		.when(editable, |row| highlighted(row, context))
		.id("add-drive");

	if !editable {
		return row;
	}

	let handle = handle.clone();

	row.cursor_pointer()
		.on_click(move |_, _, application| handle.add_drive(application))
}

fn size_text(bytes: u64) -> String {
	const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];

	let (mut tenths, mut unit) = (bytes / 100, 0);

	while tenths >= 10_000 && unit + 1 < UNITS.len() {
		tenths /= 1000;
		unit += 1;
	}

	if tenths < 100 {
		format!("{}.{} {}", tenths / 10, tenths % 10, UNITS[unit])
	} else {
		format!("{} {}", tenths / 10, UNITS[unit])
	}
}

/// `C:` is the box's own folder, so its row also shows the box's size on disk.
fn own_drive(entry: &Entry, context: &App) -> Div {
	let row = drive('c', entry.wine_box.folder.text(), context);

	match entry.usage {
		Usage::Measuring => row,
		Usage::Measured(bytes) => row.child(muted(size_text(bytes), context)),
	}
}

pub fn drives(handle: &BoxHandle, entry: &Entry, context: &App) -> Div {
	let (wine_box, editable) = (&entry.wine_box, entry.editable());

	card(context)
		.child(own_drive(entry, context))
		.children(
			wine_box
				.settings
				.drives
				.iter()
				.enumerate()
				.map(|(index, settings)| mounted(handle, index, settings, editable, context)),
		)
		.child(add_row(handle, editable, context))
}
