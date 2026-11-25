#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."

mapfile -d '' -t listed < <(git ls-files -z --cached --others --exclude-standard -- '*.rs')
files=()

for file in "${listed[@]}"; do
	if [[ -f $file ]]; then
		files+=("$file")
	fi
done

cargo run --release --quiet --manifest-path Tools/NewLines/Cargo.toml -- "${files[@]}"
