use gpui::{App, transparent_black};
use gpui_component::{ActiveTheme as _, button::ButtonCustomVariant};

/// Removing keeps the folder, so the button stays quiet until hovered.
pub fn removal(context: &App) -> ButtonCustomVariant {
	let theme = context.theme();

	ButtonCustomVariant::new(context)
		.color(transparent_black())
		.foreground(theme.muted_foreground)
		.hover(theme.danger.opacity(0.15))
		.active(theme.danger.opacity(0.25))
}
