use std::{
	collections::BTreeSet,
	fs::File,
	io::{self, BufRead as _, BufReader, Read as _},
	path::Path,
};

use super::LinkKind;

use crate::boxes::open_regular;

/// Wine registers these to hand them to the host through `winebrowser`.
const WINE_KINDS: [&str; 4] = ["ftp", "http", "https", "mailto"];

const CLASS_KEY: &[u8] = br"[Software\\Classes\\";
const URL_PROTOCOL: &[u8] = br#""URL Protocol"="#;

/// Far longer than any line that names a link kind.
const LONGEST_LINE: u64 = 4096;

fn class_kind(line: &[u8]) -> Option<LinkKind> {
	let name = line.strip_prefix(CLASS_KEY)?;
	let end = name.iter().position(|&byte| byte == b']')?;
	let kind = LinkKind::new(str::from_utf8(&name[..end]).ok()?)?;

	(!WINE_KINDS.contains(&kind.as_str())).then_some(kind)
}

fn add_kinds(mut registry: BufReader<File>, kinds: &mut BTreeSet<LinkKind>) -> io::Result<()> {
	let mut line = Vec::new();
	let mut class = None;

	loop {
		line.clear();

		if (&mut registry)
			.take(LONGEST_LINE)
			.read_until(b'\n', &mut line)?
			== 0
		{
			return Ok(());
		}
		if !line.ends_with(b"\n") {
			registry.skip_until(b'\n')?;
		}

		if line.starts_with(b"[") {
			class = class_kind(&line);
		} else if line.starts_with(URL_PROTOCOL)
			&& let Some(kind) = class.take()
		{
			kinds.insert(kind);
		}
	}
}

/// Wine opens links only through machine-wide registrations, which it keeps in `system.reg`. A box
/// whose registry the panel can't read, which its programs can cause, opens no links.
pub fn box_kinds(prefix: &Path) -> BTreeSet<LinkKind> {
	let mut kinds = BTreeSet::new();

	if let Ok(registry) = open_regular(&prefix.join("system.reg")) {
		add_kinds(BufReader::new(registry), &mut kinds).unwrap_or_default();
	}

	kinds
}
