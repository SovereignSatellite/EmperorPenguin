use gpui::{App, Div, ParentElement as _, Styled as _, div, prelude::FluentBuilder as _};
use gpui_component::{ActiveTheme as _, Disableable as _, button::Button};

use super::{icons, tuning};

use crate::{
	container::Gpu,
	interface::{
		panel::{BoxHandle, Entry},
		views::card::{card, divided, leading, muted, row},
	},
};

fn set_visible(handle: &BoxHandle, address: &str, visible: bool, context: &mut App) {
	handle.set_gpu_visible(address, visible, context);
}

fn visibility_button(
	handle: &BoxHandle,
	index: usize,
	gpu: &Gpu,
	visible: bool,
	editable: bool,
) -> Button {
	let (handle, address) = (handle.clone(), gpu.address.clone());
	let tooltip = if visible {
		"Visible to the box"
	} else {
		"Hidden from the box"
	};

	Button::new(("gpu", index))
		.outline()
		.icon(icons::visibility(visible))
		.tooltip(tooltip)
		.disabled(!editable)
		.on_click(move |_, _, context| set_visible(&handle, &address, !visible, context))
}

fn gpu_row(handle: &BoxHandle, index: usize, gpu: &Gpu, entry: &Entry, context: &App) -> Div {
	let visible = !entry.wine_box.settings.hidden_gpus.contains(&gpu.address);
	let muted_text = context.theme().muted_foreground;
	let name = div()
		.flex_1()
		.child(gpu.name.clone())
		.when(!visible, |name| name.text_color(muted_text));

	divided(row(), context)
		.child(leading(icons::gpu()))
		.child(name)
		.child(visibility_button(
			handle,
			index,
			gpu,
			visible,
			entry.editable(),
		))
}

fn sees_a_gpu(gpus: &[Gpu], entry: &Entry) -> bool {
	gpus.iter()
		.any(|gpu| !entry.wine_box.settings.hidden_gpus.contains(&gpu.address))
}

fn display_row(handle: &BoxHandle, gpus: &[Gpu], entry: &Entry, context: &App) -> Div {
	let note = (!sees_a_gpu(gpus, entry)).then(|| muted("Programs draw on the CPU.", context));

	row()
		.child(tuning::scale_button(handle, entry))
		.child(tuning::frame_limit_button(handle, entry))
		.child(div().flex_1())
		.children(note)
}

pub fn graphics(handle: &BoxHandle, gpus: &[Gpu], entry: &Entry, context: &App) -> Div {
	card(context)
		.child(display_row(handle, gpus, entry, context))
		.children(
			gpus.iter()
				.enumerate()
				.map(|(index, gpu)| gpu_row(handle, index, gpu, entry, context)),
		)
}
