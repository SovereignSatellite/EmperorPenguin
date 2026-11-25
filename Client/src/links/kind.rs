/// Holds only the characters RFC 3986 allows in a scheme, which also keeps it from breaking the list
/// of MIME types in a desktop entry.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct LinkKind(String);

const fn is_scheme_character(character: char) -> bool {
	character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
}

impl LinkKind {
	pub fn new(scheme: &str) -> Option<Self> {
		let mut characters = scheme.chars();
		let valid = characters
			.next()
			.as_ref()
			.is_some_and(char::is_ascii_alphabetic)
			&& characters.all(is_scheme_character);

		valid.then(|| Self(scheme.to_ascii_lowercase()))
	}

	pub fn of_link(link: &str) -> Option<Self> {
		let (scheme, _) = link.split_once(':')?;

		Self::new(scheme)
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	/// The host's browser opens these, while other kinds can reach any program on the host,
	/// including one that runs a file the box wrote.
	pub fn is_web(&self) -> bool {
		matches!(self.as_str(), "http" | "https")
	}
}
