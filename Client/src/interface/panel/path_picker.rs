use core::fmt::Display;

use std::path::{Path, PathBuf};

use gpui::{App, AsyncApp, Context, Entity, PathPromptOptions};

use super::ControlPanel;

type Picked<E, C> = Result<Result<Option<Vec<PathBuf>>, E>, C>;

fn receive<E, C, A>(
	panel: &mut ControlPanel,
	picked: Picked<E, C>,
	add: A,
	context: &mut Context<'_, ControlPanel>,
) where
	E: Display,
	C: Display,
	A: Fn(&mut ControlPanel, &Path, &mut Context<'_, ControlPanel>),
{
	match picked {
		Ok(Ok(Some(paths))) => {
			for path in paths {
				add(panel, &path, context);
			}
		}
		Ok(Ok(None)) => {}
		Ok(Err(error)) => panel.report(Err(error.to_string()), context),
		Err(error) => panel.report(Err(error.to_string()), context),
	}
}

async fn add_picked<R, E, C, A>(
	panel: Entity<ControlPanel>,
	picking: R,
	add: A,
	application: &mut AsyncApp,
) where
	R: Future<Output = Picked<E, C>>,
	E: Display,
	C: Display,
	A: Fn(&mut ControlPanel, &Path, &mut Context<'_, ControlPanel>),
{
	let picked = picking.await;

	panel.update(application, |panel, context| {
		receive(panel, picked, add, context);
	});
}

fn pick<A>(panel: &Entity<ControlPanel>, options: PathPromptOptions, add: A, context: &App)
where
	A: Fn(&mut ControlPanel, &Path, &mut Context<'_, ControlPanel>) + 'static,
{
	let picking = context.prompt_for_paths(options);
	let panel = panel.clone();

	context
		.spawn(async move |application| add_picked(panel, picking, add, application).await)
		.detach();
}

pub fn pick_folder<A>(panel: &Entity<ControlPanel>, add: A, context: &App)
where
	A: Fn(&mut ControlPanel, &Path, &mut Context<'_, ControlPanel>) + 'static,
{
	let options = PathPromptOptions {
		files: false,
		directories: true,
		multiple: false,
		prompt: None,
	};

	pick(panel, options, add, context);
}

pub fn pick_file<A>(panel: &Entity<ControlPanel>, add: A, context: &App)
where
	A: Fn(&mut ControlPanel, &Path, &mut Context<'_, ControlPanel>) + 'static,
{
	let options = PathPromptOptions {
		files: true,
		directories: false,
		multiple: false,
		prompt: None,
	};

	pick(panel, options, add, context);
}
