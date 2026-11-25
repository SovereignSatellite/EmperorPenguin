use std::{env, io, io::Write as _, os::unix::ffi::OsStrExt as _, os::unix::net::UnixStream};

const SOCKET: &str = concat!(
	env!("EMPEROR_PENGUIN_LINKS_FOLDER"),
	"/",
	env!("EMPEROR_PENGUIN_LINKS_SOCKET")
);

fn main() -> io::Result<()> {
	let link = env::args_os().nth(1).ok_or(io::ErrorKind::InvalidInput)?;

	UnixStream::connect(SOCKET)?.write_all(link.as_bytes())
}
