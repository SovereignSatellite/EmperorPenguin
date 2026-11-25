{ pkgs }:
let
	container = import ../Container { inherit pkgs; };
in
{
	buildInputs = with pkgs; [
		fontconfig
		freetype
		libxkbcommon
	];
	# GPUI loads these with `dlopen`, so they never appear as link dependencies.
	runtimeLibraryPath = pkgs.lib.makeLibraryPath (
		with pkgs;
		[
			vulkan-loader
			wayland
		]
	);
	# The application reads these at compile time.
	variables = {
		EMPEROR_PENGUIN_ENVIRONMENT = container.environment;
		EMPEROR_PENGUIN_FONTS = "${pkgs.geist-font}/share/fonts/opentype";
		EMPEROR_PENGUIN_LINKS_FOLDER = container.links.folder;
		EMPEROR_PENGUIN_PCI_IDS = "${pkgs.hwdata}/share/hwdata/pci.ids";
		EMPEROR_PENGUIN_LINKS_SOCKET = container.links.socket;
		EMPEROR_PENGUIN_PODMAN = pkgs.lib.getExe pkgs.podman;
		EMPEROR_PENGUIN_RESOURCES = "${../Resources}";
		EMPEROR_PENGUIN_ROOT = container.root;
		EMPEROR_PENGUIN_STORE_PATHS = container.storePaths;
		EMPEROR_PENGUIN_TRANSLATION = container.translation;
		EMPEROR_PENGUIN_WINE = "${container.wine}";
		EMPEROR_PENGUIN_UPDATE_DESKTOP_DATABASE = "${pkgs.desktop-file-utils}/bin/update-desktop-database";
	};
}
