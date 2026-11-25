mod fonts;
mod saved_mode;

use std::rc::Rc;

use gpui::{App, Window};
use gpui_component::{Theme, ThemeConfig, ThemeMode, ThemeRegistry};

pub use saved_mode::{read as saved, write as save};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
	Light,
	Dark,
	/// Follows the desktop's light or dark setting.
	Automatic,
}

fn registered(name: &str, context: &App) -> Rc<ThemeConfig> {
	Rc::clone(
		ThemeRegistry::global(context)
			.themes()
			.get(name)
			.expect("the embedded theme file names both themes"),
	)
}

fn use_vercel(theme: &mut Theme, light: Rc<ThemeConfig>, dark: Rc<ThemeConfig>) {
	theme.light_theme = light;
	theme.dark_theme = dark;
}

pub fn init(context: &mut App) {
	fonts::register(context);
	ThemeRegistry::global_mut(context)
		.load_themes_from_str(include_str!(resource!("Themes/vercel.json")))
		.expect("the embedded theme file parses");

	let (light, dark) = (
		registered("Vercel Light", context),
		registered("Vercel Dark", context),
	);

	Theme::update(context, |theme| use_vercel(theme, light, dark));
}

pub fn apply(appearance: Appearance, window: &mut Window, context: &mut App) {
	match appearance {
		Appearance::Light => Theme::change(ThemeMode::Light, Some(window), context),
		Appearance::Dark => Theme::change(ThemeMode::Dark, Some(window), context),
		Appearance::Automatic => Theme::sync_system_appearance(Some(window), context),
	}
}
