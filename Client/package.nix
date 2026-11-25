{
	pkgs,
	crane,
	environment,
}:
let
	craneLib = crane.mkLib pkgs;
	common = {
		src = pkgs.lib.fileset.toSource {
			root = ./.;
			fileset = pkgs.lib.fileset.unions [
				./Cargo.toml
				./Cargo.lock
				./src
			];
		};
		strictDeps = true;
		nativeBuildInputs = [ pkgs.pkg-config ];
		inherit (environment) buildInputs;
		doCheck = false;
	};
in
craneLib.buildPackage (
	common
	// {
		# Dependencies build once into their own package. The container's paths go only to the panel's
		# own build, so changing the container leaves them cached.
		cargoArtifacts = craneLib.buildDepsOnly common;
		nativeBuildInputs = common.nativeBuildInputs ++ [ pkgs.copyDesktopItems ];
		env = environment.variables;
		# Named after the window's app ID, so compositors and launchers match the window to this entry.
		desktopItems = [
			(pkgs.makeDesktopItem {
				name = "dev.sovereignsatellite.EmperorPenguin";
				desktopName = "Emperor Penguin";
				comment = "Run Windows programs in isolated boxes";
				exec = "emperor-penguin";
				categories = [
					"System"
					"Emulator"
				];
				startupWMClass = "dev.sovereignsatellite.EmperorPenguin";
			})
		];
		postFixup = "patchelf --add-rpath ${environment.runtimeLibraryPath} $out/bin/emperor-penguin";
	}
)
