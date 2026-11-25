{ pkgs, environment }:
pkgs.mkShell {
	nativeBuildInputs = with pkgs; [
		cargo
		clippy
		pkg-config
		rust-analyzer
		rustc
		rustfmt
	];
	inherit (environment) buildInputs;
	env = environment.variables // {
		LD_LIBRARY_PATH = environment.runtimeLibraryPath;
	};
}
