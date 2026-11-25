use gpui::{App, Div, IntoElement, ParentElement as _, Styled as _, div};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};

pub fn card(context: &App) -> Div {
	v_flex()
		.w_full()
		.bg(context.theme().group_box)
		.border_1()
		.border_color(context.theme().border)
		.rounded(context.theme().radius)
}

pub fn row() -> Div {
	h_flex().h_12().px_3().gap_3()
}

/// The first row of a card has the card's own border above it.
pub fn divided(row: Div, context: &App) -> Div {
	row.border_t_1().border_color(context.theme().border)
}

pub fn leading<E>(content: E) -> Div
where
	E: IntoElement,
{
	div()
		.w_8()
		.flex_shrink_0()
		.flex()
		.justify_center()
		.child(content)
}

pub fn muted<E>(text: E, context: &App) -> Div
where
	E: IntoElement,
{
	div()
		.text_sm()
		.text_color(context.theme().muted_foreground)
		.child(text)
}
