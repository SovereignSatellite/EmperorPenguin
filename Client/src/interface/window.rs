use gpui::{
	AnyView, App, AppContext as _, Entity, TitlebarOptions, Window, WindowDecorations,
	WindowOptions,
};
use gpui_component::{Root, TitleBar};

use super::{
	panel::{ControlPanel, Listing},
	theme::{self, Appearance},
	views::LoadFailure,
};

const APPLICATION_NAME: &str = "Emperor Penguin";

fn build_root(
	listing: Result<Listing, String>,
	window: &mut Window,
	context: &mut App,
) -> Entity<Root> {
	let saved = theme::saved();
	let appearance = saved.clone().unwrap_or(Appearance::Automatic);

	theme::apply(appearance, window, context);

	let view: AnyView = match listing {
		Ok(listing) => context
			.new(|context| ControlPanel::new(listing, appearance, saved.map(drop), window, context))
			.into(),
		Err(error) => context
			.new(|_| LoadFailure {
				error: error.into(),
			})
			.into(),
	};

	context.new(|context| Root::new(view, window, context))
}

pub fn open_window(listing: Result<Listing, String>, context: &mut App) {
	theme::init(context);

	let options = WindowOptions {
		titlebar: Some(TitlebarOptions {
			title: Some(APPLICATION_NAME.into()),
			..TitleBar::title_bar_options()
		}),
		window_decorations: Some(WindowDecorations::Client),
		app_id: Some("dev.sovereignsatellite.EmperorPenguin".into()),
		..TitleBar::window_options()
	};

	context
		.open_window(options, move |window, context| {
			build_root(listing, window, context)
		})
		.expect("failed to open the control panel window");
}
