use gpui::Context;
use gpui_component::ActiveTheme as _;

use super::{Activity, ControlPanel, background, inventory, links};

use crate::{
	boxes::{Folder, WineBox},
	container::{self, Shade, Target},
};

fn stopped(panel: &mut ControlPanel, folder: &Folder, context: &mut Context<'_, ControlPanel>) {
	if let Some(entry) = panel.entry(folder) {
		entry.activity = Activity::Stopped;

		inventory::read(&entry.wine_box, context);
	}

	links::register_link_kinds(panel, context);
	context.notify();
}

pub fn watch(wine_box: WineBox, context: &Context<'_, ControlPanel>) {
	let folder = wine_box.folder.clone();

	background::run(
		context,
		move || container::wait(&wine_box),
		move |panel, (), context| stopped(panel, &folder, context),
	);
}

/// The watch has marked the box stopped if it ended while starting.
fn ready(
	panel: &mut ControlPanel,
	folder: &Folder,
	result: Result<(), String>,
	context: &mut Context<'_, ControlPanel>,
) {
	if let Some(entry) = panel.entry(folder)
		&& entry.activity == Activity::Starting
	{
		entry.activity = Activity::Running;
	}

	panel.report(result, context);
}

fn launch_in_box(wine_box: WineBox, target: Target, context: &Context<'_, ControlPanel>) {
	let folder = wine_box.folder.clone();

	background::run(
		context,
		move || container::launch(&wine_box, &target),
		move |panel, result, context| ready(panel, &folder, result, context),
	);
}

fn started(
	panel: &mut ControlPanel,
	wine_box: WineBox,
	target: Target,
	result: Result<(), String>,
	context: &mut Context<'_, ControlPanel>,
) {
	if result.is_ok() {
		watch(wine_box.clone(), context);
		launch_in_box(wine_box, target, context);
	} else if let Some(entry) = panel.entry(&wine_box.folder) {
		entry.activity = Activity::Stopped;
	}

	panel.report(result, context);
}

/// The box takes the panel's light or dark colors as it starts.
fn start(wine_box: WineBox, target: Target, context: &Context<'_, ControlPanel>) {
	let starting = wine_box.clone();
	let shade = if context.theme().mode.is_dark() {
		Shade::Dark
	} else {
		Shade::Light
	};

	background::run(
		context,
		move || container::start(&starting, shade),
		move |panel, result, context| started(panel, wine_box, target, result, context),
	);
}

/// A launch while the box starts or stops is dropped.
pub fn launch(
	panel: &mut ControlPanel,
	folder: &Folder,
	target: Target,
	context: &mut Context<'_, ControlPanel>,
) {
	let Some(entry) = panel.entry(folder) else {
		return;
	};
	let wine_box = entry.wine_box.clone();

	match entry.activity {
		Activity::Stopped => {
			entry.activity = Activity::Starting;

			start(wine_box, target, context);
		}
		Activity::Running => launch_in_box(wine_box, target, context),
		Activity::Starting | Activity::Stopping => {}
	}

	context.notify();
}

/// The box's watch marks it stopped.
pub fn stop(panel: &mut ControlPanel, index: usize, context: &mut Context<'_, ControlPanel>) {
	let entry = &mut panel.entries[index];
	let wine_box = entry.wine_box.clone();

	entry.activity = Activity::Stopping;

	background::run(context, move || container::stop(&wine_box), |_, (), _| {});
	context.notify();
}
