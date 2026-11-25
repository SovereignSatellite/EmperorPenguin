use gpui_component::Icon;

use crate::boxes::{Access, Switch};

fn icon(svg: &[u8]) -> Icon {
	Icon::default().data(svg)
}

pub fn switch(switch: Switch, enabled: bool) -> Icon {
	icon(match (switch, enabled) {
		(Switch::Network, true) => include_bytes!(resource!("Icons/network.svg")).as_slice(),
		(Switch::Network, false) => include_bytes!(resource!("Icons/network-off.svg")).as_slice(),
		(Switch::Sound, true) => include_bytes!(resource!("Icons/sound.svg")).as_slice(),
		(Switch::Sound, false) => include_bytes!(resource!("Icons/sound-off.svg")).as_slice(),
		(Switch::Microphone, true) => include_bytes!(resource!("Icons/microphone.svg")).as_slice(),
		(Switch::Microphone, false) => {
			include_bytes!(resource!("Icons/microphone-off.svg")).as_slice()
		}
	})
}

pub fn access(access: Access) -> Icon {
	icon(match access {
		Access::ReadOnly => include_bytes!(resource!("Icons/read-only.svg")).as_slice(),
		Access::ReadWrite => include_bytes!(resource!("Icons/read-write.svg")).as_slice(),
	})
}

pub fn program() -> Icon {
	icon(include_bytes!(resource!("Icons/program.svg")))
}

pub fn scale() -> Icon {
	icon(include_bytes!(resource!("Icons/scale.svg")))
}

pub fn frame_rate() -> Icon {
	icon(include_bytes!(resource!("Icons/frame-rate.svg")))
}

pub fn run() -> Icon {
	icon(include_bytes!(resource!("Icons/run.svg")))
}

pub fn tools() -> Icon {
	icon(include_bytes!(resource!("Icons/tools.svg")))
}

pub fn automatic_appearance() -> Icon {
	icon(include_bytes!(resource!("Icons/automatic-appearance.svg")))
}

pub fn gpu() -> Icon {
	icon(include_bytes!(resource!("Icons/gpu.svg")))
}

pub fn visibility(visible: bool) -> Icon {
	icon(if visible {
		include_bytes!(resource!("Icons/visible.svg")).as_slice()
	} else {
		include_bytes!(resource!("Icons/hidden.svg")).as_slice()
	})
}
