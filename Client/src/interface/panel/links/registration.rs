use std::path::PathBuf;

use gpui::Context;

use crate::{
	interface::panel::{ControlPanel, background},
	links,
};

/// A box's registry runs to megabytes, on a disk that may be busy.
pub fn register_link_kinds(panel: &ControlPanel, context: &Context<'_, ControlPanel>) {
	let prefixes: Vec<PathBuf> = panel
		.entries
		.iter()
		.map(|entry| entry.wine_box.prefix())
		.collect();

	background::run(
		context,
		move || links::register(&prefixes),
		ControlPanel::report,
	);
}
