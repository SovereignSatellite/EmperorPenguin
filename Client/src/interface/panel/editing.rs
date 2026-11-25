use core::num::NonZeroU32;

use std::path::Path;

use gpui::Context;

use super::{ControlPanel, Entry, background, links, listing};

use crate::{
	boxes::{self, Access, DRIVE_LETTERS, Drive, Folder, Settings, Switch, WineBox},
	container::Running,
};

fn new_drive(drives: &[Drive], folder: Folder, access: Access) -> Result<Drive, String> {
	for letter in DRIVE_LETTERS {
		if drives.iter().all(|drive| drive.letter != letter) {
			return Ok(Drive {
				letter,
				folder,
				access,
			});
		}
	}

	Err("Every drive letter is in use.".into())
}

fn settings(panel: &mut ControlPanel, index: usize) -> &mut Settings {
	&mut panel.entries[index].wine_box.settings
}

fn save(panel: &mut ControlPanel, index: usize, context: &mut Context<'_, ControlPanel>) {
	let result = panel.entries[index].wine_box.save();

	panel.report(result, context);
}

fn save_list(panel: &ControlPanel, context: &Context<'_, ControlPanel>) -> Result<(), String> {
	let readable = panel
		.entries
		.iter()
		.map(|entry| entry.wine_box.folder.text());
	let unreadable = panel.unreadable.iter().map(|listed| listed.line.as_str());
	let result = boxes::save(readable.chain(unreadable));

	links::register_link_kinds(panel, context);

	result
}

pub fn set_switch(
	panel: &mut ControlPanel,
	index: usize,
	switch: Switch,
	enabled: bool,
	context: &mut Context<'_, ControlPanel>,
) {
	*settings(panel, index).switch_mut(switch) = enabled;

	save(panel, index, context);
}

pub fn set_scale(
	panel: &mut ControlPanel,
	index: usize,
	scale: NonZeroU32,
	context: &mut Context<'_, ControlPanel>,
) {
	settings(panel, index).scale = scale;

	save(panel, index, context);
}

pub fn set_frame_limit(
	panel: &mut ControlPanel,
	index: usize,
	limit: Option<NonZeroU32>,
	context: &mut Context<'_, ControlPanel>,
) {
	settings(panel, index).frame_limit = limit;

	save(panel, index, context);
}

pub fn set_gpu_visible(
	panel: &mut ControlPanel,
	index: usize,
	address: &str,
	visible: bool,
	context: &mut Context<'_, ControlPanel>,
) {
	let hidden = &mut settings(panel, index).hidden_gpus;

	hidden.retain(|gpu| gpu != address);

	if !visible {
		hidden.push(address.to_owned());
	}

	save(panel, index, context);
}

pub fn set_access(
	panel: &mut ControlPanel,
	index: usize,
	drive: usize,
	access: Access,
	context: &mut Context<'_, ControlPanel>,
) {
	let mut shared = settings(panel, index).drives[drive].clone();

	shared.access = access;

	if let Err(error) = boxes::check_drive(&shared) {
		panel.report(Err(error), context);

		return;
	}

	settings(panel, index).drives[drive] = shared;

	save(panel, index, context);
}

pub fn remove_drive(
	panel: &mut ControlPanel,
	index: usize,
	drive: usize,
	context: &mut Context<'_, ControlPanel>,
) {
	settings(panel, index).drives.remove(drive);
	save(panel, index, context);
}

fn add_to(entry: &mut Entry, path: &Path) -> Result<(), String> {
	if !entry.editable() {
		return Err("Stop the box to change settings.".into());
	}

	let folder = Folder::new(path)?;
	let access = if boxes::holds_a_box(&folder)? {
		Access::ReadOnly
	} else {
		Access::ReadWrite
	};
	let drive = new_drive(&entry.wine_box.settings.drives, folder, access)?;

	entry.wine_box.settings.drives.push(drive);

	entry.wine_box.save()
}

/// The folder picker answers later, by when the box may have started or left the list.
pub fn add_drive(
	panel: &mut ControlPanel,
	folder: &Folder,
	path: &Path,
	context: &mut Context<'_, ControlPanel>,
) {
	let Some(entry) = panel.entry(folder) else {
		return;
	};
	let result = add_to(entry, path);

	panel.report(result, context);
}

fn open(folder: Folder) -> Result<(WineBox, bool), String> {
	let wine_box = WineBox::open(folder)?;

	Ok(listing::paired(&Running::query()?, wine_box))
}

fn list(
	panel: &mut ControlPanel,
	paired: (WineBox, bool),
	context: &Context<'_, ControlPanel>,
) -> Result<(), String> {
	if panel.lists(&paired.0.folder) {
		return Err("That box is already listed.".into());
	}

	panel.entries.push(listing::entry(paired, context));

	panel.selected = panel.entries.len() - 1;

	save_list(panel, context)
}

fn opened(
	panel: &mut ControlPanel,
	opening: Result<(WineBox, bool), String>,
	context: &mut Context<'_, ControlPanel>,
) {
	let result = opening.and_then(|paired| list(panel, paired, context));

	panel.report(result, context);
}

pub fn add_box(panel: &mut ControlPanel, path: &Path, context: &mut Context<'_, ControlPanel>) {
	match Folder::new(path) {
		Ok(folder) => background::run(context, move || open(folder), opened),
		Err(error) => panel.report(Err(error), context),
	}
}

pub fn remove_selected(panel: &mut ControlPanel, context: &mut Context<'_, ControlPanel>) {
	match panel.selected.checked_sub(panel.entries.len()) {
		None => drop(panel.entries.remove(panel.selected)),
		Some(unreadable) => drop(panel.unreadable.remove(unreadable)),
	}

	let listed = panel.entries.len() + panel.unreadable.len();

	panel.selected = panel.selected.min(listed.saturating_sub(1));

	let result = save_list(panel, context);

	panel.report(result, context);
}
