use std::env;

use gpui::{App, Div, FontWeight, ParentElement as _, Styled as _, div};
use gpui_component::{ActiveTheme as _, h_flex};

fn separator(context: &App) -> Div {
	div()
		.flex_shrink_0()
		.text_color(context.theme().muted_foreground.opacity(0.5))
		.child("/")
}

fn parent(name: &str, context: &App) -> Div {
	div()
		.flex_shrink_0()
		.text_color(context.theme().muted_foreground)
		.child(name.to_owned())
}

fn separated_parent(name: &str, context: &App) -> [Div; 2] {
	[separator(context), parent(name, context)]
}

fn inside_home<'text>(text: &'text str, home: &str) -> Option<&'text str> {
	text.strip_prefix(home).filter(|rest| rest.starts_with('/'))
}

fn home_relative(text: &str) -> String {
	let home = env::var("HOME");
	let inside = home
		.as_deref()
		.ok()
		.and_then(|home| inside_home(text, home));

	inside.map_or_else(|| text.to_owned(), |rest| format!("~{rest}"))
}

/// The parent folders overflow on the left, so the folder's own name always stays in view.
pub fn path(text: &str, context: &App) -> Div {
	let text = home_relative(text);
	let (parents, name) = text.rsplit_once('/').unwrap_or(("", &text));
	let mut folders = parents.split('/');
	let root = folders.next().expect("splitting yields at least one piece");

	h_flex()
		.flex_1()
		.min_w_0()
		.gap_1()
		.font_family(context.theme().mono_font_family.clone())
		.text_sm()
		.child(
			h_flex()
				.min_w_0()
				.overflow_hidden()
				.justify_end()
				.gap_1()
				.child(parent(root, context))
				.children(folders.flat_map(|folder| separated_parent(folder, context))),
		)
		.child(separator(context))
		.child(
			div()
				.flex_shrink_0()
				.font_weight(FontWeight::MEDIUM)
				.child(name.to_owned()),
		)
}
