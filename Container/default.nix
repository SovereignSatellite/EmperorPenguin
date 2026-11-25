{ pkgs }:
let
	links = {
		folder = "/run/emperor-penguin";
		socket = "links";
	};
	# Wine's `winebrowser` opens links with the first program it finds from a list that starts with
	# `xdg-open`.
	xdg-open =
		pkgs.runCommandCC "emperor-penguin-xdg-open"
			{
				nativeBuildInputs = [ pkgs.rustc ];
				EMPEROR_PENGUIN_LINKS_FOLDER = links.folder;
				EMPEROR_PENGUIN_LINKS_SOCKET = links.socket;
			}
			''
				mkdir -p $out/bin
				rustc --edition 2024 -C opt-level=s -C strip=symbols -o $out/bin/xdg-open ${./xdg-open.rs}
			'';
	wine = import ./Wine { inherit pkgs; };
	programs = pkgs.buildEnv {
		name = "emperor-penguin-programs";
		paths = with pkgs; [
			mesa
			wine
			xdg-open
		];
	};
	translation = import ./translation.nix { inherit pkgs; };
	fonts = pkgs.makeFontsConf {
		fontDirectories = [
			pkgs.geist-font
			pkgs.liberation_ttf
		];
	};
	# Mount points for the container's own file systems, and the mtab link Podman checks for before
	# mounting them, all of which the read-only root cannot gain later.
	root = pkgs.runCommand "emperor-penguin-root" { } ''
		mkdir -p $out/{dev,etc,nix/store,prefix,proc,run,sys,tmp,usr,var}
		ln -s /proc/mounts $out/etc/mtab
		ln -s ${programs}/{bin,lib,share} $out/
	'';
	environment = pkgs.replaceVars ../Resources/Container/environment { inherit fonts; };
in
{
	inherit
		root
		environment
		translation
		links
		wine
		;
	storePaths = "${
		pkgs.closureInfo {
			rootPaths = [
				root
				environment
				translation
			];
		}
	}/store-paths";
}
