use gpui::{Entity, Styled as _};
use gpui_component::{IconName, button::Button};

use crate::interface::panel::{self, ControlPanel};

pub fn add_button(panel: &Entity<ControlPanel>) -> Button {
	let panel = panel.clone();

	Button::new("add-box")
		.outline()
		.w_full()
		.icon(IconName::Plus)
		.label("Add box")
		.on_click(move |_, _, context| panel::pick_folder(&panel, panel::add_box, context))
}
