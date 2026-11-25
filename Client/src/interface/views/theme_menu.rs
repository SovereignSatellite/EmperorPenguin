use gpui::{App, Entity, IntoElement, Window};
use gpui_component::{
	Icon, IconName,
	button::{Button, ButtonVariants as _},
	menu::{DropdownMenu as _, PopupMenu, PopupMenuItem},
};

use super::icons;

use crate::interface::{panel::ControlPanel, theme::Appearance};

const APPEARANCES: [(Appearance, &str); 3] = [
	(Appearance::Light, "Light"),
	(Appearance::Dark, "Dark"),
	(Appearance::Automatic, "Auto"),
];

fn icon(appearance: Appearance) -> Icon {
	match appearance {
		Appearance::Light => IconName::Sun.into(),
		Appearance::Dark => IconName::Moon.into(),
		Appearance::Automatic => icons::automatic_appearance(),
	}
}

fn choose(
	panel: &Entity<ControlPanel>,
	appearance: Appearance,
	window: &mut Window,
	context: &mut App,
) {
	panel.update(context, |panel, context| {
		panel.set_appearance(appearance, window, context);
	});
}

fn menu(mut menu: PopupMenu, panel: &Entity<ControlPanel>, current: Appearance) -> PopupMenu {
	for (appearance, label) in APPEARANCES {
		let panel = panel.clone();

		menu = menu.item(
			PopupMenuItem::new(label)
				.checked(appearance == current)
				.on_click(move |_, window, context| choose(&panel, appearance, window, context)),
		);
	}

	menu
}

pub fn theme_menu(panel: &Entity<ControlPanel>, current: Appearance) -> impl IntoElement {
	let panel = panel.clone();

	Button::new("theme")
		.ghost()
		.icon(icon(current))
		.tooltip("Theme")
		.dropdown_menu(move |popup, _, _| menu(popup, &panel, current))
}
