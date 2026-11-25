use std::{
	io::{self, Read as _},
	os::unix::net::UnixStream,
};

/// Every frame starts with its payload length, channel, two offset words and flags.
const DESCRIPTOR: usize = 20;
const CONTROL_CHANNEL: u32 = u32::MAX;
/// `PipeWire`'s `PulseAudio` server refuses longer frames too.
const LONGEST_PAYLOAD: u32 = 16 * 1024 * 1024;
const UNSIGNED_TAG: u8 = b'L';
const AUTH_COMMAND: u32 = 8;
const ERROR_COMMAND: u32 = 0;
const ACCESS_ERROR: u32 = 1;
/// A stream with this property can pose as a host device, such as a microphone other programs record.
const DEVICE_CLASS_KEY: &[u8] = b"tmedia.class\0";

pub struct Request {
	pub command: u32,
	pub tag: u32,
}

fn word(bytes: &[u8], at: usize) -> u32 {
	let mut word = [0; 4];

	word.copy_from_slice(&bytes[at..at + 4]);

	u32::from_be_bytes(word)
}

fn invalid() -> io::Error {
	io::ErrorKind::InvalidData.into()
}

/// Leaves one whole frame, descriptor included, in `frame`.
pub fn read_frame(stream: &mut UnixStream, frame: &mut Vec<u8>) -> io::Result<()> {
	frame.resize(DESCRIPTOR, 0);
	stream.read_exact(frame)?;

	let length = word(frame, 0);

	if length > LONGEST_PAYLOAD {
		return Err(invalid());
	}

	// The frame grows only as its payload arrives, since a client can announce a long one and stall.
	let mut payload = stream.by_ref().take(u64::from(length));

	payload.read_to_end(frame)?;

	if payload.limit() != 0 {
		return Err(io::ErrorKind::UnexpectedEof.into());
	}

	Ok(())
}

/// A client offers shared memory in the upper half of its version word, and a server that accepts
/// passes audio in file descriptors, which the filter doesn't forward.
fn decline_shared_memory(auth: &mut [u8]) -> io::Result<()> {
	auth.get_mut(DESCRIPTOR + 11..DESCRIPTOR + 13)
		.ok_or_else(invalid)?
		.fill(0);

	Ok(())
}

/// Stream data carries no request, and a control frame always starts with its command and tag.
pub fn request(frame: &mut [u8]) -> io::Result<Option<Request>> {
	if word(frame, 4) != CONTROL_CHANNEL {
		return Ok(None);
	}

	let request = match &frame[DESCRIPTOR..] {
		[UNSIGNED_TAG, _, _, _, _, UNSIGNED_TAG, _, _, _, _, ..] => Request {
			command: word(frame, DESCRIPTOR + 1),
			tag: word(frame, DESCRIPTOR + 6),
		},
		_ => return Err(invalid()),
	};

	if request.command == AUTH_COMMAND {
		decline_shared_memory(frame)?;
	}

	Ok(Some(request))
}

pub fn sets_device_class(frame: &[u8]) -> bool {
	frame
		.windows(DEVICE_CLASS_KEY.len())
		.any(|window| window == DEVICE_CLASS_KEY)
}

impl Request {
	pub fn refusal(&self) -> Vec<u8> {
		let mut frame = Vec::with_capacity(DESCRIPTOR + 15);

		for descriptor_word in [15, CONTROL_CHANNEL, 0, 0, 0] {
			frame.extend(u32::to_be_bytes(descriptor_word));
		}

		for payload_word in [ERROR_COMMAND, self.tag, ACCESS_ERROR] {
			frame.push(UNSIGNED_TAG);
			frame.extend(payload_word.to_be_bytes());
		}

		frame
	}
}
