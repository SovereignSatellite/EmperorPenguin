use gpui::Context;

use super::{Activity, ControlPanel, Entry, Usage, inventory, session};

use crate::{
	boxes::{self, Unreadable, WineBox},
	container::{self, Gpu, Running},
};

pub struct Listing {
	pub boxes: Vec<(WineBox, bool)>,
	pub unreadable: Vec<Unreadable>,
	pub gpus: Vec<Gpu>,
}

pub fn load() -> Result<Listing, String> {
	let (boxes, unreadable) = boxes::load()?;
	let running = Running::query()?;
	let boxes = boxes
		.into_iter()
		.map(|wine_box| paired(&running, wine_box))
		.collect();

	Ok(Listing {
		boxes,
		unreadable,
		gpus: container::gpus(),
	})
}

pub fn paired(running: &Running, wine_box: WineBox) -> (WineBox, bool) {
	let runs = running.contains(&wine_box);

	(wine_box, runs)
}

pub fn entry((wine_box, runs): (WineBox, bool), context: &Context<'_, ControlPanel>) -> Entry {
	let activity = if runs {
		session::watch(wine_box.clone(), context);

		Activity::Running
	} else {
		Activity::Stopped
	};

	inventory::read(&wine_box, context);

	Entry {
		wine_box,
		activity,
		programs: Vec::new(),
		usage: Usage::Measuring,
	}
}
