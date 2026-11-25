//! The Emperor Penguin control panel, its link handler and its boxes' sound filter.

/// A file's path in the repository's `Resources` folder, as Nix copies it to the store.
macro_rules! resource {
	($file:literal) => {
		concat!(env!("EMPEROR_PENGUIN_RESOURCES"), "/", $file)
	};
}

mod boxes;
mod container;
mod folders;
mod interface;
mod links;
mod programs;
mod sound;

use std::env;

use gpui::App;
use gpui_kit_assets::Assets;

fn start(listing: Result<interface::Listing, String>, context: &mut App) {
	gpui_component::init(context);
	context
		.on_window_closed(|context, _| context.quit())
		.detach();
	interface::open_window(listing, context);
}

fn main() -> Result<(), String> {
	let mut arguments = env::args_os().skip(1);

	match arguments.next() {
		Some(first) if first == sound::FILTER => return sound::filter(arguments),
		Some(link) => return links::deliver(&link),
		None => {}
	}

	let listing = interface::load();

	gpui_platform::application()
		.with_assets(Assets)
		.run(move |context| start(listing, context));

	Ok(())
}
