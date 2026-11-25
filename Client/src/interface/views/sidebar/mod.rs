mod add_box;

use gpui::{App, Div, Entity, Hsla, ParentElement as _, Styled as _, div};
use gpui_component::{
	ActiveTheme as _, Icon, IconName, Sizable as _,
	button::{Button, ButtonVariants as _},
	h_flex,
	sidebar::{Sidebar, SidebarMenu, SidebarMenuItem},
};

use add_box::add_button;

use super::removal::removal;

use crate::{
	boxes::Unreadable,
	interface::panel::{self, Activity, ControlPanel, Entry},
};

#[derive(Clone, Copy)]
enum Status {
	Stopped,
	Busy,
	Running,
	Unreadable,
}

fn dot(color: Hsla) -> Div {
	div().size_1p5().rounded_full().bg(color)
}

impl Status {
	const fn is_removable(self) -> bool {
		match self {
			Self::Stopped | Self::Unreadable => true,
			Self::Busy | Self::Running => false,
		}
	}

	fn mark(self, context: &App) -> Option<Div> {
		match self {
			Self::Stopped => None,
			Self::Busy => Some(dot(context.theme().warning)),
			Self::Running => Some(dot(context.theme().success)),
			Self::Unreadable => Some(
				div().child(
					Icon::new(IconName::TriangleAlert)
						.small()
						.text_color(context.theme().warning),
				),
			),
		}
	}
}

struct Listed<'panel> {
	name: &'panel str,
	status: Status,
}

impl<'panel> Listed<'panel> {
	fn from_entry(entry: &'panel Entry) -> Self {
		Listed {
			name: entry.wine_box.folder.name(),
			status: match entry.activity {
				Activity::Stopped => Status::Stopped,
				Activity::Starting | Activity::Stopping => Status::Busy,
				Activity::Running => Status::Running,
			},
		}
	}

	fn from_unreadable(listed: &'panel Unreadable) -> Self {
		Listed {
			name: listed.name(),
			status: Status::Unreadable,
		}
	}
}

fn select(panel: &Entity<ControlPanel>, index: usize, context: &mut App) {
	panel.update(context, |panel, context| panel.select(index, context));
}

/// Stops the click from reaching the item, which would select the removed index.
fn remove_selected(panel: &Entity<ControlPanel>, context: &mut App) {
	context.stop_propagation();
	panel.update(context, panel::remove_selected);
}

fn removable_suffix(panel: &Entity<ControlPanel>, status: Status, context: &App) -> Div {
	let panel = panel.clone();

	h_flex()
		.gap_2()
		.child(
			Button::new("remove-listed")
				.custom(removal(context))
				.xsmall()
				.icon(IconName::Close)
				.tooltip("Remove from list")
				.on_click(move |_, _, context| remove_selected(&panel, context)),
		)
		.children(status.mark(context))
}

fn item(
	panel: &Entity<ControlPanel>,
	selected: usize,
	index: usize,
	listed: &Listed<'_>,
) -> SidebarMenuItem {
	let (selecting, status) = (panel.clone(), listed.status);
	let item = SidebarMenuItem::new(listed.name.to_owned())
		.active(index == selected)
		.on_click(move |_, _, context| select(&selecting, index, context));

	if index == selected && status.is_removable() {
		let removing = panel.clone();

		item.suffix(move |_, context| removable_suffix(&removing, status, context))
	} else {
		item.suffix(move |_, context| div().children(status.mark(context)))
	}
}

pub fn sidebar(panel: &Entity<ControlPanel>, control_panel: &ControlPanel) -> Sidebar<SidebarMenu> {
	let boxes = control_panel.entries.iter().map(Listed::from_entry);
	let unreadable = control_panel.unreadable.iter().map(Listed::from_unreadable);
	let selected = control_panel.selected;
	let menu = SidebarMenu::new().children(
		boxes
			.chain(unreadable)
			.enumerate()
			.map(|(index, listed)| item(panel, selected, index, &listed)),
	);

	Sidebar::new("boxes")
		.w_48()
		.header(add_button(panel))
		.child(menu)
}
