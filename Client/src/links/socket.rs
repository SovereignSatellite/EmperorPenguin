use core::time::Duration;

use std::{
	fs,
	io::{self, Read as _},
	os::unix::net::{UnixListener, UnixStream},
	path::PathBuf,
	time::Instant,
};

use crate::folders;

pub const BOX_FOLDER: &str = env!("EMPEROR_PENGUIN_LINKS_FOLDER");
const SOCKET: &str = env!("EMPEROR_PENGUIN_LINKS_SOCKET");

const LONGEST_LINK: usize = 64 * 1024;
const SENDING_TIME: Duration = Duration::from_secs(5);

/// Boxes mount this folder rather than the socket in it, so they still reach the socket that a later
/// start of the control panel puts in its place.
pub fn host_folder() -> Result<PathBuf, String> {
	let folder = folders::runtime()?.join("emperor-penguin");

	fs::create_dir_all(&folder).map_err(|error| error.to_string())?;

	Ok(folder)
}

pub struct Links {
	listener: UnixListener,
	received: Vec<u8>,
}

fn read_link(mut stream: UnixStream, link: &mut Vec<u8>) -> io::Result<()> {
	let deadline = Instant::now() + SENDING_TIME;
	let mut chunk = [0; 4096];

	link.clear();

	loop {
		// The standard library refuses a zero timeout, which the socket would take as none.
		let left = deadline
			.checked_duration_since(Instant::now())
			.filter(|left| !left.is_zero())
			.ok_or(io::ErrorKind::TimedOut)?;

		stream.set_read_timeout(Some(left))?;

		let read = stream.read(&mut chunk)?;

		if read == 0 {
			return Ok(());
		}

		link.extend_from_slice(&chunk[..read]);

		if link.len() > LONGEST_LINK {
			return Err(io::ErrorKind::FileTooLarge.into());
		}
	}
}

impl Links {
	/// A socket that refuses connections is left from an earlier start and can't be bound again, so it
	/// is replaced. One that accepts them belongs to a running control panel, which keeps it.
	pub fn listen() -> Result<Self, String> {
		let socket = host_folder()?.join(SOCKET);

		if UnixStream::connect(&socket).is_ok() {
			return Err("another control panel receives them".into());
		}

		folders::remove_if_present(&socket).map_err(|error| error.to_string())?;

		let listener = UnixListener::bind(&socket).map_err(|error| error.to_string())?;

		Ok(Self {
			listener,
			received: Vec::new(),
		})
	}

	/// Another control panel's check that this one is running sends nothing.
	pub fn receive(&mut self) -> io::Result<String> {
		loop {
			let (stream, _) = self.listener.accept()?;

			if read_link(stream, &mut self.received).is_ok()
				&& !self.received.is_empty()
				&& let Ok(link) = str::from_utf8(&self.received)
			{
				return Ok(link.to_owned());
			}
		}
	}
}
