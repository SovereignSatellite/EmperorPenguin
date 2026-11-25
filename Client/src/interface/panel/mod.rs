mod background;
mod box_handle;
mod editing;
mod inventory;
mod links;
mod listing;
mod path_picker;
mod session;

use std::path::Path;

use gpui::{ClickEvent, Context, SharedString, Window};

pub use box_handle::BoxHandle;
pub use editing::{add_box, remove_selected};
pub use inventory::Program;
pub use listing::{Listing, load};
pub use path_picker::pick_folder;

use crate::{
	boxes::{Folder, Unreadable, WineBox},
	container::Gpu,
	interface::theme::{self, Appearance},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Activity {
	Stopped,
	Starting,
	Running,
	Stopping,
}

#[derive(Clone, Copy)]
pub enum Usage {
	Measuring,
	Measured(u64),
}

pub struct Entry {
	pub wine_box: WineBox,
	pub activity: Activity,
	pub programs: Vec<Program>,
	pub usage: Usage,
}

impl Entry {
	/// Settings apply when the box starts, and a running box stays listed.
	pub fn editable(&self) -> bool {
		self.activity == Activity::Stopped
	}

	pub const fn busy(&self) -> bool {
		matches!(self.activity, Activity::Starting | Activity::Stopping)
	}
}

pub struct ControlPanel {
	pub entries: Vec<Entry>,
	pub unreadable: Vec<Unreadable>,
	/// The host's, which change only with its hardware.
	pub gpus: Vec<Gpu>,
	pub selected: usize,
	pub message: Option<SharedString>,
	pub appearance: Appearance,
}

impl ControlPanel {
	pub fn new(
		listing: Listing,
		appearance: Appearance,
		theme: Result<(), String>,
		window: &mut Window,
		context: &Context<'_, Self>,
	) -> Self {
		let panel = Self {
			entries: listing
				.boxes
				.into_iter()
				.map(|paired| listing::entry(paired, context))
				.collect(),
			unreadable: listing.unreadable,
			gpus: listing.gpus,
			selected: 0,
			message: theme.err().map(SharedString::from),
			appearance,
		};

		context
			.observe_window_appearance(window, Self::follow_desktop)
			.detach();
		links::forward_links(context);
		links::register_link_kinds(&panel, context);

		panel
	}

	fn follow_desktop(&mut self, window: &mut Window, context: &mut Context<'_, Self>) {
		if self.appearance == Appearance::Automatic {
			theme::apply(Appearance::Automatic, window, context);
		}
	}

	pub fn set_appearance(
		&mut self,
		appearance: Appearance,
		window: &mut Window,
		context: &mut Context<'_, Self>,
	) {
		self.appearance = appearance;

		theme::apply(appearance, window, context);
		self.report(theme::save(appearance), context);
	}

	/// A success leaves the message alone, since it may report an unrelated failure.
	pub fn report(&mut self, result: Result<(), String>, context: &mut Context<'_, Self>) {
		if let Err(error) = result {
			self.message = Some(error.into());

			context.notify();
		}
	}

	pub fn select(&mut self, index: usize, context: &mut Context<'_, Self>) {
		self.selected = index;

		context.notify();
	}

	pub fn dismiss(
		&mut self,
		_event: &ClickEvent,
		_window: &mut Window,
		context: &mut Context<'_, Self>,
	) {
		self.message = None;

		context.notify();
	}

	fn entry(&mut self, folder: &Folder) -> Option<&mut Entry> {
		self.entries
			.iter_mut()
			.find(|entry| entry.wine_box.folder == *folder)
	}

	fn lists(&self, folder: &Folder) -> bool {
		self.entries
			.iter()
			.any(|entry| entry.wine_box.folder == *folder)
			|| self.unreadable.iter().any(|listed| {
				Folder::new(Path::new(&listed.line)).is_ok_and(|listed| listed == *folder)
			})
	}
}
