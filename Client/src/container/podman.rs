use std::{
	ffi::OsStr,
	io,
	os::{fd::OwnedFd, unix::process::CommandExt as _},
	process::{Command, Output},
};

use rustix::io::{FdFlags, fcntl_setfd};

const PODMAN: &str = env!("EMPEROR_PENGUIN_PODMAN");

fn finish(output: io::Result<Output>) -> Result<String, String> {
	let output = output.map_err(|error| error.to_string())?;

	if !output.status.success() {
		return Err(format!(
			"Podman failed with {}: {}",
			output.status,
			String::from_utf8_lossy(&output.stderr).trim()
		));
	}

	Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn run<I, S>(arguments: I) -> Result<String, String>
where
	I: IntoIterator<Item = S>,
	S: AsRef<OsStr>,
{
	finish(Command::new(PODMAN).args(arguments).output())
}

fn inherit(files: &[OwnedFd]) -> io::Result<()> {
	for file in files {
		fcntl_setfd(file, FdFlags::empty())?;
	}

	Ok(())
}

/// Podman passes `held` on to the container only if Podman itself inherits them, and the panel opens
/// every file closed on exec.
pub fn run_holding<I, S>(arguments: I, held: Vec<OwnedFd>) -> Result<String, String>
where
	I: IntoIterator<Item = S>,
	S: AsRef<OsStr>,
{
	let mut command = Command::new(PODMAN);

	command.args(arguments);

	// SAFETY: `inherit` only calls fcntl, which is async-signal-safe, on files the closure owns.
	unsafe {
		command.pre_exec(move || inherit(&held));
	}

	finish(command.output())
}

pub fn attempt<I, S>(arguments: I)
where
	I: IntoIterator<Item = S>,
	S: AsRef<OsStr>,
{
	run(arguments).map(drop).unwrap_or_default();
}
