{
	description = "Isolated Windows boxes on Linux";

	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
		crane.url = "github:ipetkov/crane";
	};

	outputs =
		{ nixpkgs, crane, ... }:
		let
			pkgs = nixpkgs.legacyPackages.x86_64-linux;
			environment = import ./Client/environment.nix { inherit pkgs; };
		in
		{
			packages.x86_64-linux.default = import ./Client/package.nix { inherit pkgs crane environment; };
			devShells.x86_64-linux.default = import ./Client/shell.nix { inherit pkgs environment; };
		};
}
