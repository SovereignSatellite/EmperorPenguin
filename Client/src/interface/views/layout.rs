use gpui::{
	App, Context, Div, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Render,
	Stateful, StatefulInteractiveElement as _, Styled as _, Window, div,
};
use gpui_component::{ActiveTheme as _, TitleBar, alert::Alert, h_flex, v_flex};

use super::{
	box_page::box_page,
	card::{card, muted},
	sidebar::sidebar,
	theme_menu::theme_menu,
	unreadable::unreadable_view,
};

use crate::interface::panel::ControlPanel;

fn empty_state(context: &App) -> Div {
	card(context).p_3().child(muted(
		"A box is a folder with its own Windows. Add box creates one in an empty folder, or lists a \
			 box folder you already have.",
		context,
	))
}

/// The list is empty when the selection is past its end.
fn content(panel: &Entity<ControlPanel>, control_panel: &ControlPanel, context: &App) -> Div {
	let selected = control_panel.selected;

	match selected.checked_sub(control_panel.entries.len()) {
		None => box_page(
			panel,
			selected,
			&control_panel.entries[selected],
			&control_panel.gpus,
			context,
		),
		Some(index) => control_panel.unreadable.get(index).map_or_else(
			|| empty_state(context),
			|listed| unreadable_view(listed, context),
		),
	}
}

/// Messages float over the page, so they move nothing.
fn floating(alert: Alert, context: &App) -> Stateful<Div> {
	div()
		.id("message")
		.max_h_40()
		.overflow_y_scroll()
		.absolute()
		.bottom_3()
		.left_3()
		.right_3()
		.occlude()
		.rounded(context.theme().radius)
		.bg(context.theme().background)
		.child(alert)
}

impl Render for ControlPanel {
	fn render(
		&mut self,
		_window: &mut Window,
		context: &mut Context<'_, Self>,
	) -> impl IntoElement {
		let panel = context.entity();
		let page = v_flex()
			.size_full()
			.relative()
			.child(
				TitleBar::new().child(
					h_flex()
						.w_full()
						.justify_end()
						.pr_2()
						.child(theme_menu(&panel, self.appearance)),
				),
			)
			.child(
				h_flex()
					.flex_1()
					.min_h_0()
					.items_start()
					.child(sidebar(&panel, self))
					.child(div().flex_1().p_3().child(content(&panel, self, context))),
			);
		let Some(message) = self.message.clone() else {
			return page;
		};
		let alert = Alert::error("message", message).on_close(context.listener(Self::dismiss));

		page.child(floating(alert, context))
	}
}
