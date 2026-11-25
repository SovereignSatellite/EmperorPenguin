//! `PipeWire`'s `PulseAudio` server can't enforce a box's sound switches itself, because it takes
//! access levels from properties the client sends.

mod protocol;

use core::sync::atomic::{AtomicUsize, Ordering};

use std::{
	env,
	ffi::OsString,
	fs::File,
	io::{self, Read as _, Write as _},
	os::{
		fd::{AsRawFd as _, FromRawFd as _, OwnedFd, RawFd},
		unix::{
			net::{UnixListener, UnixStream},
			process::CommandExt as _,
		},
	},
	path::Path,
	process::{Command, Stdio},
	sync::Arc,
	thread,
};

use rustix::{
	event::{PollFd, PollFlags, poll},
	io::{FdFlags, fcntl_setfd},
	process::setsid,
};

use crate::folders;

pub const FILTER: &str = "--filter-sound";
const PLAYBACK: &str = "--playback";
const RECORDING: &str = "--recording";
const MOST_CONNECTIONS: usize = 16;

/// What Wine needs to connect and list devices.
const CONNECTION_COMMANDS: [u32; 9] = [
	8,  // AUTH
	9,  // SET_CLIENT_NAME
	20, // GET_SERVER_INFO
	21, // GET_SINK_INFO
	22, // GET_SINK_INFO_LIST
	23, // GET_SOURCE_INFO
	24, // GET_SOURCE_INFO_LIST
	82, // UPDATE_CLIENT_PROPLIST
	85, // REMOVE_CLIENT_PROPLIST
];

/// Like `RECORDING_COMMANDS`, each of these acts only on a stream the client created.
const PLAYBACK_COMMANDS: [u32; 13] = [
	3,  // CREATE_PLAYBACK_STREAM
	4,  // DELETE_PLAYBACK_STREAM
	12, // DRAIN_PLAYBACK_STREAM
	14, // GET_PLAYBACK_LATENCY
	41, // CORK_PLAYBACK_STREAM
	42, // FLUSH_PLAYBACK_STREAM
	43, // TRIGGER_PLAYBACK_STREAM
	46, // SET_PLAYBACK_STREAM_NAME
	60, // PREBUF_PLAYBACK_STREAM
	72, // SET_PLAYBACK_STREAM_BUFFER_ATTR
	74, // UPDATE_PLAYBACK_STREAM_SAMPLE_RATE
	81, // UPDATE_PLAYBACK_STREAM_PROPLIST
	84, // REMOVE_PLAYBACK_STREAM_PROPLIST
];

const RECORDING_COMMANDS: [u32; 10] = [
	5,  // CREATE_RECORD_STREAM
	6,  // DELETE_RECORD_STREAM
	47, // SET_RECORD_STREAM_NAME
	57, // GET_RECORD_LATENCY
	58, // CORK_RECORD_STREAM
	59, // FLUSH_RECORD_STREAM
	73, // SET_RECORD_STREAM_BUFFER_ATTR
	75, // UPDATE_RECORD_STREAM_SAMPLE_RATE
	80, // UPDATE_RECORD_STREAM_PROPLIST
	83, // REMOVE_RECORD_STREAM_PROPLIST
];

#[derive(Clone, Copy)]
pub struct Grant {
	pub playback: bool,
	pub recording: bool,
}

impl Grant {
	/// The refused commands include loading a module, which can write any host file, changing devices
	/// or other programs' streams, and listing other programs.
	fn allows(self, command: u32) -> bool {
		CONNECTION_COMMANDS.contains(&command)
			|| (self.playback && PLAYBACK_COMMANDS.contains(&command))
			|| (self.recording && RECORDING_COMMANDS.contains(&command))
	}
}

fn is_ready(side: &PollFd<'_>) -> bool {
	!side.revents().is_empty()
}

fn ready(client: &UnixStream, server: &UnixStream) -> io::Result<[bool; 2]> {
	let mut sides = [
		PollFd::new(client, PollFlags::IN),
		PollFd::new(server, PollFlags::IN),
	];

	poll(&mut sides, None)?;

	Ok(sides.each_ref().map(is_ready))
}

fn forward_request(
	client: &mut UnixStream,
	server: &mut UnixStream,
	frame: &mut Vec<u8>,
	grant: Grant,
) -> io::Result<()> {
	protocol::read_frame(client, frame)?;

	match protocol::request(frame)? {
		Some(request) if !grant.allows(request.command) || protocol::sets_device_class(frame) => {
			client.write_all(&request.refusal())
		}
		_ => server.write_all(frame),
	}
}

fn relay(mut client: UnixStream, upstream: &Path, grant: Grant) -> io::Result<()> {
	let mut server = UnixStream::connect(upstream)?;
	let mut frame = Vec::new();

	loop {
		let [from_client, from_server] = ready(&client, &server)?;

		if from_client {
			forward_request(&mut client, &mut server, &mut frame, grant)?;
		}

		if from_server {
			protocol::read_frame(&mut server, &mut frame)?;
			client.write_all(&frame)?;
		}
	}
}

fn relay_counted(client: UnixStream, upstream: &Path, grant: Grant, open: &AtomicUsize) {
	relay(client, upstream, grant).unwrap_or_default();
	open.fetch_sub(1, Ordering::Relaxed);
}

/// The host's server takes only a few dozen clients in all, so one box may hold only some of them.
fn serve(listener: &UnixListener, upstream: &Path, grant: Grant) {
	let open = Arc::new(AtomicUsize::new(0));

	for client in listener.incoming().flatten() {
		if open.load(Ordering::Relaxed) == MOST_CONNECTIONS {
			continue;
		}

		open.fetch_add(1, Ordering::Relaxed);

		let (upstream, open) = (upstream.to_owned(), Arc::clone(&open));

		thread::spawn(move || relay_counted(client, &upstream, grant, &open));
	}
}

fn descriptor(argument: Option<OsString>) -> Option<RawFd> {
	argument?.to_str()?.parse().ok()
}

/// Runs as its own process, started by `start`, so a box keeps its sound after the control panel
/// exits.
pub fn filter<I>(mut arguments: I) -> Result<(), String>
where
	I: Iterator<Item = OsString>,
{
	let (Some(listener), Some(box_pipe)) =
		(descriptor(arguments.next()), descriptor(arguments.next()))
	else {
		return Err("The sound filter needs its socket and pipe.".into());
	};
	let mut grant = Grant {
		playback: false,
		recording: false,
	};

	for argument in arguments {
		grant.playback |= argument == PLAYBACK;
		grant.recording |= argument == RECORDING;
	}

	// SAFETY: `start` passes this descriptor open and gives it to nothing else.
	let listener = unsafe { UnixListener::from_raw_fd(listener) };
	// SAFETY: as above.
	let mut box_pipe = unsafe { File::from_raw_fd(box_pipe) };
	let upstream = folders::runtime()?.join("pulse/native");

	thread::spawn(move || serve(&listener, &upstream, grant));
	box_pipe.read(&mut [0]).map(drop).unwrap_or_default();

	Ok(())
}

/// Its own session keeps the filter running when the terminal that started the control panel
/// interrupts it.
fn detach(listener: &OwnedFd, box_pipe: &OwnedFd) -> io::Result<()> {
	setsid()?;

	for file in [listener, box_pipe] {
		fcntl_setfd(file, FdFlags::empty())?;
	}

	Ok(())
}

fn spawn(listener: OwnedFd, box_pipe: OwnedFd, grant: Grant) -> io::Result<()> {
	let mut command = Command::new(env::current_exe()?);

	command
		.arg(FILTER)
		.arg(listener.as_raw_fd().to_string())
		.arg(box_pipe.as_raw_fd().to_string())
		.stdin(Stdio::null())
		.stdout(Stdio::null());

	if grant.playback {
		command.arg(PLAYBACK);
	}
	if grant.recording {
		command.arg(RECORDING);
	}

	// SAFETY: `detach` only calls setsid and fcntl, which are async-signal-safe, on files the closure
	// owns.
	unsafe {
		command.pre_exec(move || detach(&listener, &box_pipe));
	}

	let mut child = command.spawn()?;

	thread::spawn(move || child.wait());

	Ok(())
}

/// Returns the pipe end that the box holds while the filter serves it at `socket`.
pub fn start(socket: &Path, grant: Grant) -> Result<OwnedFd, String> {
	folders::remove_if_present(socket).map_err(|error| error.to_string())?;

	let listener = UnixListener::bind(socket).map_err(|error| error.to_string())?;
	let (box_pipe, close) = io::pipe().map_err(|error| error.to_string())?;

	spawn(listener.into(), box_pipe.into(), grant).map_err(|error| error.to_string())?;

	Ok(close.into())
}
