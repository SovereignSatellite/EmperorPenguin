use std::ffi::OsStr;

use super::{LinkKind, registry};

use crate::{boxes, container};

pub fn deliver(link: &OsStr) -> Result<(), String> {
	let link = link.to_str().ok_or("Links must be text.")?;
	let kind = LinkKind::of_link(link).ok_or_else(|| format!("{link} is not a link."))?;
	let (boxes, _) = boxes::load()?;
	let running = container::Running::query()?;
	let opener = boxes
		.iter()
		.filter(|wine_box| running.contains(wine_box))
		.find(|wine_box| registry::box_kinds(&wine_box.prefix()).contains(&kind))
		.ok_or_else(|| format!("No running box opens {} links.", kind.as_str()))?;

	container::open(opener, link)
}
