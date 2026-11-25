use std::{path::PathBuf, sync::Arc};

use gpui::{Context, Image, ImageFormat, SharedString};

use super::{ControlPanel, Entry, Usage, background};

use crate::{
	boxes::{self, Folder, WineBox},
	programs,
};

pub struct Program {
	pub name: SharedString,
	pub icon: Option<Arc<Image>>,
	pub shortcut: PathBuf,
}

impl From<programs::Program> for Program {
	fn from(program: programs::Program) -> Self {
		Self {
			name: program.name.into(),
			icon: program
				.icon
				.map(|bytes| Arc::new(Image::from_bytes(ImageFormat::Ico, bytes))),
			shortcut: program.shortcut,
		}
	}
}

fn installed(wine_box: &WineBox) -> Vec<Program> {
	programs::installed(wine_box)
		.into_iter()
		.map(Program::from)
		.collect()
}

fn update<T, A>(
	panel: &mut ControlPanel,
	folder: &Folder,
	result: Result<T, String>,
	apply: A,
	context: &mut Context<'_, ControlPanel>,
) where
	A: FnOnce(&mut Entry, T),
{
	match result {
		Ok(value) => {
			if let Some(entry) = panel.entry(folder) {
				apply(entry, value);
			}

			context.notify();
		}
		Err(error) => panel.report(Err(error), context),
	}
}

fn set_programs(entry: &mut Entry, programs: Vec<Program>) {
	entry.programs = programs;
}

const fn set_usage(entry: &mut Entry, bytes: u64) {
	entry.usage = Usage::Measured(bytes);
}

/// A box's programs and size change only while it runs.
pub fn read(wine_box: &WineBox, context: &Context<'_, ControlPanel>) {
	let (listing, measuring) = (wine_box.clone(), wine_box.folder.clone());
	let (listed, measured) = (wine_box.folder.clone(), wine_box.folder.clone());

	background::run(
		context,
		move || installed(&listing),
		move |panel, programs, context| update(panel, &listed, Ok(programs), set_programs, context),
	);
	background::run(
		context,
		move || boxes::disk_usage(measuring.path()).map_err(|error| error.to_string()),
		move |panel, result, context| update(panel, &measured, result, set_usage, context),
	);
}
