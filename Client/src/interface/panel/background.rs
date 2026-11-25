use gpui::{AsyncApp, Context, Entity, Task};

use super::ControlPanel;

async fn complete<T, F>(
	panel: Entity<ControlPanel>,
	task: Task<T>,
	finish: F,
	application: &mut AsyncApp,
) where
	T: 'static,
	F: FnOnce(&mut ControlPanel, T, &mut Context<'_, ControlPanel>),
{
	let value = task.await;

	panel.update(application, |panel, context| finish(panel, value, context));
}

pub fn run<T, W, F>(context: &Context<'_, ControlPanel>, work: W, finish: F)
where
	T: Send + 'static,
	W: FnOnce() -> T + Send + 'static,
	F: FnOnce(&mut ControlPanel, T, &mut Context<'_, ControlPanel>) + 'static,
{
	let task = context.background_executor().spawn(async move { work() });
	let panel = context.entity();

	context
		.spawn(async move |_, application| complete(panel, task, finish, application).await)
		.detach();
}
