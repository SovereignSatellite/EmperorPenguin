use core::{num::NonZeroU32, ops::RangeInclusive, str::Lines};

use std::path::Path;

use super::Folder;

/// `C:` is the box's own drive.
pub const DRIVE_LETTERS: RangeInclusive<char> = 'd'..='y';

/// The range Windows offers, in percent.
const SCALE_RANGE: RangeInclusive<u32> = 100..=500;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Access {
	ReadOnly,
	ReadWrite,
}

#[derive(Clone)]
pub struct Drive {
	pub letter: char,
	pub folder: Folder,
	pub access: Access,
}

#[derive(Clone, Copy)]
pub enum Switch {
	Network,
	Sound,
	Microphone,
}

#[derive(Clone)]
pub struct Settings {
	pub network: bool,
	pub sound: bool,
	pub microphone: bool,
	/// In percent.
	pub scale: NonZeroU32,
	/// In frames per second.
	pub frame_limit: Option<NonZeroU32>,
	/// PCI addresses, so a GPU added later is visible.
	pub hidden_gpus: Vec<String>,
	pub drives: Vec<Drive>,
}

const fn switch_word(enabled: bool) -> &'static str {
	if enabled { "on" } else { "off" }
}

fn parse_switch(word: &str) -> Result<bool, String> {
	match word {
		"on" => Ok(true),
		"off" => Ok(false),
		_ => Err(format!("expected on or off, found {word}")),
	}
}

fn parse_count(word: &str, unit: &str) -> Result<NonZeroU32, String> {
	word.parse()
		.map_err(|error| format!("expected a count of {unit}, found {word}: {error}"))
}

fn parse_scale(word: &str) -> Result<NonZeroU32, String> {
	let percent = word
		.strip_suffix('%')
		.ok_or_else(|| format!("expected a percentage, found {word}"))?;
	let scale = parse_count(percent, "percent")?;

	if !SCALE_RANGE.contains(&scale.get()) {
		let (lowest, highest) = SCALE_RANGE.into_inner();

		return Err(format!(
			"expected a scale from {lowest}% to {highest}%, found {word}"
		));
	}

	Ok(scale)
}

fn parse_frame_limit(word: &str) -> Result<Option<NonZeroU32>, String> {
	if word == "off" {
		return Ok(None);
	}

	parse_count(word, "frames per second").map(Some)
}

fn frame_limit_word(limit: Option<NonZeroU32>) -> String {
	limit.map_or_else(|| "off".to_owned(), |frames| frames.to_string())
}

fn parse_gpus(word: &str) -> Vec<String> {
	if word == "none" {
		return Vec::new();
	}

	word.split(',').map(str::to_owned).collect()
}

fn gpus_word(gpus: &[String]) -> String {
	if gpus.is_empty() {
		return "none".to_owned();
	}

	gpus.join(",")
}

fn parse_letter(word: &str) -> Result<char, String> {
	let mut letters = word.chars();

	match (letters.next(), letters.next()) {
		(Some(letter), None) if DRIVE_LETTERS.contains(&letter) => Ok(letter),
		_ => Err(format!("expected a drive letter from d to y, found {word}")),
	}
}

fn value<'text>(lines: &mut Lines<'text>, key: &str) -> Result<&'text str, String> {
	let line = lines
		.next()
		.ok_or_else(|| format!("expected the {key} setting"))?;

	line.strip_prefix(key)
		.and_then(|rest| rest.strip_prefix(' '))
		.ok_or_else(|| format!("expected the {key} setting, found {line}"))
}

impl Access {
	fn parse(word: &str) -> Result<Self, String> {
		match word {
			"read-only" => Ok(Self::ReadOnly),
			"read-write" => Ok(Self::ReadWrite),
			_ => Err(format!("expected read-only or read-write, found {word}")),
		}
	}

	const fn serialize(self) -> &'static str {
		match self {
			Self::ReadOnly => "read-only",
			Self::ReadWrite => "read-write",
		}
	}
}

impl Drive {
	fn parse(line: &str) -> Result<Self, String> {
		let mut fields = line.splitn(4, ' ');
		let (Some("drive"), Some(letter), Some(access), Some(folder)) =
			(fields.next(), fields.next(), fields.next(), fields.next())
		else {
			return Err(format!("expected a drive, found {line}"));
		};

		Ok(Self {
			letter: parse_letter(letter)?,
			folder: Folder::new(Path::new(folder))?,
			access: Access::parse(access)?,
		})
	}

	fn serialize(&self) -> String {
		format!(
			"drive {} {} {}\n",
			self.letter,
			self.access.serialize(),
			self.folder.text()
		)
	}
}

impl Settings {
	pub const fn new() -> Self {
		Self {
			network: false,
			sound: true,
			microphone: false,
			scale: NonZeroU32::new(100).unwrap(),
			frame_limit: None,
			hidden_gpus: Vec::new(),
			drives: Vec::new(),
		}
	}

	pub fn parse(text: &str) -> Result<Self, String> {
		let mut lines = text.lines();

		Ok(Self {
			network: parse_switch(value(&mut lines, "network")?)?,
			sound: parse_switch(value(&mut lines, "sound")?)?,
			microphone: parse_switch(value(&mut lines, "microphone")?)?,
			scale: parse_scale(value(&mut lines, "scale")?)?,
			frame_limit: parse_frame_limit(value(&mut lines, "frame-limit")?)?,
			hidden_gpus: parse_gpus(value(&mut lines, "hidden-gpus")?),
			drives: lines.map(Drive::parse).collect::<Result<_, _>>()?,
		})
	}

	pub fn serialize(&self) -> String {
		let drives: String = self.drives.iter().map(Drive::serialize).collect();

		format!(
			"network {}\nsound {}\nmicrophone {}\nscale {}%\nframe-limit {}\nhidden-gpus {}\n{drives}",
			switch_word(self.network),
			switch_word(self.sound),
			switch_word(self.microphone),
			self.scale,
			frame_limit_word(self.frame_limit),
			gpus_word(&self.hidden_gpus),
		)
	}

	pub const fn switch(&self, switch: Switch) -> bool {
		match switch {
			Switch::Network => self.network,
			Switch::Sound => self.sound,
			Switch::Microphone => self.microphone,
		}
	}

	pub const fn switch_mut(&mut self, switch: Switch) -> &mut bool {
		match switch {
			Switch::Network => &mut self.network,
			Switch::Sound => &mut self.sound,
			Switch::Microphone => &mut self.microphone,
		}
	}
}
