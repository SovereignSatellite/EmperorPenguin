use core::num::NonZeroU32;

use std::path::Path;

use gpui::{App, Context, Entity};

use super::{ControlPanel, editing, path_picker, session};

use crate::{
	boxes::{Access, Folder, Switch},
	container::Target,
};

#[derive(Clone)]
pub struct BoxHandle {
	pub panel: Entity<ControlPanel>,
	pub index: usize,
}

fn drive_adder(
	folder: Folder,
) -> impl Fn(&mut ControlPanel, &Path, &mut Context<'_, ControlPanel>) + 'static {
	move |panel, path, context| editing::add_drive(panel, &folder, path, context)
}

fn file_runner(
	folder: Folder,
) -> impl Fn(&mut ControlPanel, &Path, &mut Context<'_, ControlPanel>) + 'static {
	move |panel, file, context| run_file(panel, &folder, file, context)
}

fn run_file(
	panel: &mut ControlPanel,
	folder: &Folder,
	file: &Path,
	context: &mut Context<'_, ControlPanel>,
) {
	session::launch(panel, folder, Target::File(file.to_owned()), context);
}

impl BoxHandle {
	fn folder(&self, context: &App) -> Folder {
		self.panel.read(context).entries[self.index]
			.wine_box
			.folder
			.clone()
	}

	pub fn launch(&self, target: Target, context: &mut App) {
		let folder = self.folder(context);

		self.panel.update(context, |panel, context| {
			session::launch(panel, &folder, target, context);
		});
	}

	pub fn run_file(&self, context: &App) {
		path_picker::pick_file(&self.panel, file_runner(self.folder(context)), context);
	}

	pub fn stop(&self, context: &mut App) {
		self.panel.update(context, |panel, context| {
			session::stop(panel, self.index, context);
		});
	}

	pub fn set_switch(&self, switch: Switch, enabled: bool, context: &mut App) {
		self.panel.update(context, |panel, context| {
			editing::set_switch(panel, self.index, switch, enabled, context);
		});
	}

	pub fn set_scale(&self, scale: NonZeroU32, context: &mut App) {
		self.panel.update(context, |panel, context| {
			editing::set_scale(panel, self.index, scale, context);
		});
	}

	pub fn set_frame_limit(&self, limit: Option<NonZeroU32>, context: &mut App) {
		self.panel.update(context, |panel, context| {
			editing::set_frame_limit(panel, self.index, limit, context);
		});
	}

	pub fn set_gpu_visible(&self, address: &str, visible: bool, context: &mut App) {
		self.panel.update(context, |panel, context| {
			editing::set_gpu_visible(panel, self.index, address, visible, context);
		});
	}

	pub fn set_access(&self, drive: usize, access: Access, context: &mut App) {
		self.panel.update(context, |panel, context| {
			editing::set_access(panel, self.index, drive, access, context);
		});
	}

	pub fn remove_drive(&self, drive: usize, context: &mut App) {
		self.panel.update(context, |panel, context| {
			editing::remove_drive(panel, self.index, drive, context);
		});
	}

	pub fn add_drive(&self, context: &App) {
		path_picker::pick_folder(&self.panel, drive_adder(self.folder(context)), context);
	}
}
