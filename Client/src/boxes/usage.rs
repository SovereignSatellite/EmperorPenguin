use std::{fs, io, os::unix::fs::MetadataExt as _, path::Path};

const BLOCK_SIZE: u64 = 512;

fn present<T>(result: io::Result<T>) -> io::Result<Option<T>> {
	match result {
		Ok(value) => Ok(Some(value)),
		Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
		Err(error) => Err(error),
	}
}

/// Counts allocated blocks without following symbolic links. A running box may delete files while
/// this walks its folder.
pub fn disk_usage(folder: &Path) -> io::Result<u64> {
	let mut folders = vec![folder.to_path_buf()];
	let mut bytes = 0;

	while let Some(current) = folders.pop() {
		let Some(entries) = present(fs::read_dir(&current))? else {
			continue;
		};

		for entry in entries {
			let entry = entry?;
			let Some(metadata) = present(entry.metadata())? else {
				continue;
			};

			bytes += metadata.blocks() * BLOCK_SIZE;

			if metadata.is_dir() {
				folders.push(entry.path());
			}
		}
	}

	Ok(bytes)
}
