use std::io;

use gpui::{AsyncApp, Context, Entity};

use super::register_link_kinds;

use crate::{
	interface::panel::ControlPanel,
	links::{LinkKind, Links},
};

fn receive(mut links: Links) -> (Links, io::Result<String>) {
	let link = links.receive();

	(links, link)
}

/// A program registers its link kind as it installs, and often sends a link out right after, such
/// as a sign-in page that returns through that kind.
fn opened(panel: &mut ControlPanel, context: &mut Context<'_, ControlPanel>, link: &str) {
	if !LinkKind::of_link(link).is_some_and(|kind| kind.is_web()) {
		panel.report(Err("Boxes can only open web links.".into()), context);

		return;
	}

	context.open_url(link);
	register_link_kinds(panel, context);
}

async fn forward(panel: &Entity<ControlPanel>, application: &mut AsyncApp) -> String {
	let mut links = match Links::listen() {
		Ok(links) => links,
		Err(error) => return error,
	};

	loop {
		let (returned, received) = application
			.background_executor()
			.spawn(async move { receive(links) })
			.await;

		links = returned;

		match received {
			Ok(link) => panel.update(application, |panel, context| opened(panel, context, &link)),
			Err(error) => return error.to_string(),
		}
	}
}

async fn report_end(panel: Entity<ControlPanel>, application: &mut AsyncApp) {
	let error = forward(&panel, application).await;
	let message = format!("This window doesn't receive links that boxes open: {error}");

	panel.update(application, |panel, context| {
		panel.report(Err(message), context);
	});
}

pub fn forward_links(context: &Context<'_, ControlPanel>) {
	let panel = context.entity();

	context
		.spawn(async move |_, application| report_end(panel, application).await)
		.detach();
}
