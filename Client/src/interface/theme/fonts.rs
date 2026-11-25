use std::borrow::Cow;

use gpui::App;

macro_rules! font {
	($file:literal) => {
		Cow::Borrowed(include_bytes!(concat!(env!("EMPEROR_PENGUIN_FONTS"), "/", $file)).as_slice())
	};
}

pub fn register(context: &App) {
	context
		.text_system()
		.add_fonts(vec![
			font!("Geist-Regular.otf"),
			font!("Geist-Medium.otf"),
			font!("GeistMono-Regular.otf"),
		])
		.expect("the bundled fonts load");
}
