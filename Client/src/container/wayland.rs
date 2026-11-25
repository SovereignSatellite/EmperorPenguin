use core::fmt::Display;

use std::{
	io,
	os::{
		fd::{AsFd as _, OwnedFd},
		unix::net::UnixListener,
	},
	path::Path,
};

use wayland_client::{
	Connection, Dispatch, QueueHandle, delegate_noop,
	globals::{GlobalListContents, registry_queue_init},
	protocol::wl_registry::{self, WlRegistry},
};
use wayland_protocols::wp::security_context::v1::client::{
	wp_security_context_manager_v1::WpSecurityContextManagerV1,
	wp_security_context_v1::WpSecurityContextV1,
};

use crate::folders;

const SANDBOX_ENGINE: &str = "dev.sovereignsatellite.EmperorPenguin";

struct Requests;

impl Dispatch<WlRegistry, GlobalListContents> for Requests {
	fn event(
		_state: &mut Self,
		_registry: &WlRegistry,
		_event: wl_registry::Event,
		_globals: &GlobalListContents,
		_connection: &Connection,
		_queue: &QueueHandle<Self>,
	) {
	}
}

delegate_noop!(Requests: WpSecurityContextManagerV1);
delegate_noop!(Requests: WpSecurityContextV1);

fn failed<E>(error: E) -> String
where
	E: Display,
{
	format!("Wayland failed: {error}")
}

fn listen(socket: &Path) -> Result<UnixListener, String> {
	folders::remove_if_present(socket).map_err(|error| error.to_string())?;

	UnixListener::bind(socket).map_err(|error| error.to_string())
}

/// The compositor serves a socket made this way without the protocols that read the screen, the
/// clipboard or other programs' input, and stops once every copy of the returned pipe end closes.
pub fn filtered_socket(socket: &Path, instance: &str) -> Result<OwnedFd, String> {
	let connection = Connection::connect_to_env().map_err(failed)?;
	let (globals, mut queue) = registry_queue_init::<Requests>(&connection).map_err(failed)?;
	let manager: WpSecurityContextManagerV1 =
		globals.bind(&queue.handle(), 1..=1, ()).map_err(|error| {
			format!("The compositor doesn't filter Wayland sockets, which boxes need: {error}")
		})?;
	let listener = listen(socket)?;
	let (closed, close) = io::pipe().map_err(|error| error.to_string())?;

	let context = manager.create_listener(listener.as_fd(), closed.as_fd(), &queue.handle(), ());

	context.set_sandbox_engine(SANDBOX_ENGINE.to_owned());
	context.set_instance_id(instance.to_owned());
	context.commit();
	queue.roundtrip(&mut Requests).map_err(failed)?;

	Ok(close.into())
}
