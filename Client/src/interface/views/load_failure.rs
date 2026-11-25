use gpui::{Context, IntoElement, ParentElement as _, Render, SharedString, Styled as _, Window};
use gpui_component::{TitleBar, alert::Alert, v_flex};

/// Replaces the control panel when the box list cannot be read, so no change overwrites the list.
pub struct LoadFailure {
	pub error: SharedString,
}

impl Render for LoadFailure {
	fn render(
		&mut self,
		_window: &mut Window,
		_context: &mut Context<'_, Self>,
	) -> impl IntoElement {
		v_flex()
			.size_full()
			.child(TitleBar::new())
			.child(Alert::error("load-failure", self.error.clone()))
	}
}
