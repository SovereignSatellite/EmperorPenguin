mod drives;
mod graphics;
mod programs;
mod tools;
mod tuning;

use gpui::{App, Div, Entity, ParentElement as _, Styled as _, div, prelude::FluentBuilder as _};
use gpui_component::{
	Disableable as _, IconName,
	button::{Button, ButtonVariants as _},
	v_flex,
};

use drives::drives;
use programs::programs;

use super::{
	card::{card, muted, row},
	icons,
};

use crate::{
	boxes::Switch,
	container::Gpu,
	interface::panel::{Activity, BoxHandle, ControlPanel, Entry},
};

fn run_button(handle: &BoxHandle, busy: bool) -> Button {
	let handle = handle.clone();

	Button::new("run")
		.primary()
		.icon(icons::run())
		.label("Run…")
		.tooltip("Run a program or installer")
		.disabled(busy)
		.on_click(move |_, _, context| handle.run_file(context))
}

fn stop_button(handle: &BoxHandle, activity: Activity) -> Option<Button> {
	if activity == Activity::Stopped {
		return None;
	}

	let handle = handle.clone();
	let button = Button::new("stop")
		.outline()
		.icon(IconName::Square)
		.tooltip("Stop")
		.loading(activity != Activity::Running)
		.on_click(move |_, _, context| handle.stop(context));

	Some(button)
}

const fn switch_tooltip(switch: Switch, enabled: bool) -> &'static str {
	match (switch, enabled) {
		(Switch::Network, true) => "Network on",
		(Switch::Network, false) => "Network off",
		(Switch::Sound, true) => "Sound on",
		(Switch::Sound, false) => "Sound off",
		(Switch::Microphone, true) => "Microphone on",
		(Switch::Microphone, false) => "Microphone off",
	}
}

fn switch_button(handle: &BoxHandle, entry: &Entry, switch: Switch) -> Button {
	let enabled = entry.wine_box.settings.switch(switch);
	let handle = handle.clone();

	Button::new(("switch", switch as usize))
		.outline()
		.icon(icons::switch(switch, enabled))
		.tooltip(switch_tooltip(switch, enabled))
		.disabled(!entry.editable())
		.on_click(move |_, _, context| handle.set_switch(switch, !enabled, context))
}

const fn note(activity: Activity) -> Option<&'static str> {
	match activity {
		Activity::Stopped => None,
		Activity::Starting => Some("Starting Windows. The first start takes longer."),
		Activity::Running | Activity::Stopping => Some("Stop the box to change settings."),
	}
}

fn locked(region: Div, editable: bool) -> Div {
	region.when(!editable, |region| region.opacity(0.4))
}

/// The note fills the toolbar's free space, so showing it moves nothing.
fn note_space(activity: Activity, context: &App) -> Div {
	let space = div().flex_1().min_w_0();

	match note(activity) {
		Some(text) => space.child(muted(text, context).truncate()),
		None => space,
	}
}

fn session_card(handle: &BoxHandle, entry: &Entry, context: &App) -> Div {
	let busy = entry.busy();

	card(context).child(
		row()
			.child(run_button(handle, busy))
			.child(tools::tools_button(handle, busy))
			.children(stop_button(handle, entry.activity))
			.child(note_space(entry.activity, context))
			.child(switch_button(handle, entry, Switch::Network))
			.child(switch_button(handle, entry, Switch::Sound))
			.child(switch_button(handle, entry, Switch::Microphone)),
	)
}

pub fn box_page(
	panel: &Entity<ControlPanel>,
	index: usize,
	entry: &Entry,
	gpus: &[Gpu],
	context: &App,
) -> Div {
	let handle = BoxHandle {
		panel: panel.clone(),
		index,
	};

	v_flex()
		.gap_3()
		.child(session_card(&handle, entry, context))
		.child(programs(&handle, entry, context))
		.child(locked(
			graphics::graphics(&handle, gpus, entry, context),
			entry.editable(),
		))
		.child(locked(drives(&handle, entry, context), entry.editable()))
}
