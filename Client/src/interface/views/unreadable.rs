use gpui::{App, Div, ParentElement as _, Styled as _, div};
use gpui_component::{ActiveTheme as _, Icon, IconName};

use super::{
	card::{card, leading, row},
	path::path,
};

use crate::boxes::Unreadable;

pub fn unreadable_view(listed: &Unreadable, context: &App) -> Div {
	let warning = Icon::new(IconName::TriangleAlert).text_color(context.theme().warning);

	card(context).child(
		row()
			.child(leading(warning))
			.child(path(&listed.line, context))
			.child(
				div()
					.text_sm()
					.text_color(context.theme().muted_foreground)
					.child(listed.error.clone()),
			),
	)
}
