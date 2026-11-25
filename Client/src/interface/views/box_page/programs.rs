use std::{path::Path, sync::Arc};

use gpui::{
	AnyElement, AnyView, App, Div, Hsla, InteractiveElement as _, IntoElement as _,
	ParentElement as _, SharedString, Stateful, StatefulInteractiveElement as _, Styled as _,
	StyledImage as _, Window, div, img,
};
use gpui_component::{ActiveTheme as _, h_flex, tooltip::Tooltip, v_flex};

use super::icons;

use crate::{
	container::Target,
	interface::{
		panel::{BoxHandle, Entry, Program},
		views::card::{card, muted, row},
	},
};

fn placeholder(color: Hsla) -> AnyElement {
	icons::program()
		.size_8()
		.text_color(color)
		.into_any_element()
}

fn picture(program: &Program, context: &App) -> AnyElement {
	let color = context.theme().muted_foreground;

	match &program.icon {
		Some(image) => img(Arc::clone(image))
			.size_8()
			.with_fallback(move || placeholder(color))
			.into_any_element(),
		None => placeholder(color),
	}
}

fn launch(handle: &BoxHandle, shortcut: &Path, context: &mut App) {
	handle.launch(Target::File(shortcut.to_owned()), context);
}

fn name_tooltip(name: &SharedString, window: &mut Window, context: &mut App) -> AnyView {
	Tooltip::new(name.clone()).build(window, context)
}

fn launchable(
	tile: Stateful<Div>,
	handle: &BoxHandle,
	program: &Program,
	context: &App,
) -> Stateful<Div> {
	let (handle, shortcut) = (handle.clone(), program.shortcut.clone());
	let highlight = context.theme().accent;

	tile.cursor_pointer()
		.hover(move |tile| tile.bg(highlight))
		.on_click(move |_, _, application| launch(&handle, &shortcut, application))
}

/// A launch while the box starts or stops would be dropped, so tiles rest until it settles.
fn tile(
	handle: &BoxHandle,
	index: usize,
	program: &Program,
	busy: bool,
	context: &App,
) -> Stateful<Div> {
	let name = program.name.clone();
	let tile = v_flex()
		.id(("program", index))
		.w_24()
		.items_center()
		.gap_1p5()
		.p_1p5()
		.rounded_md()
		.tooltip(move |window, application| name_tooltip(&name, window, application))
		.child(picture(program, context))
		.child(
			div()
				.w_full()
				.text_xs()
				.text_center()
				.line_clamp(2)
				.text_ellipsis()
				.child(program.name.clone()),
		);

	if busy {
		return tile.opacity(0.4);
	}

	launchable(tile, handle, program, context)
}

pub fn programs(handle: &BoxHandle, entry: &Entry, context: &App) -> Div {
	if entry.programs.is_empty() {
		let hint = "Programs installed in this box show up here. Run an installer to add one.";

		return card(context).child(row().child(muted(hint, context)));
	}

	card(context).child(
		h_flex().flex_wrap().items_start().gap_1().p_2().children(
			entry
				.programs
				.iter()
				.enumerate()
				.map(|(index, program)| tile(handle, index, program, entry.busy(), context)),
		),
	)
}
