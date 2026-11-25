{ pkgs }:
let
	vkd3d-proton = pkgs.fetchurl {
		url = "https://github.com/HansKristian-Work/vkd3d-proton/releases/download/v3.1/vkd3d-proton-3.1.tar.zst";
		hash = "sha256-uF6oIZtxCTo5JJE916rmFyMz3AwGmYPtUQgW1n8CEgs=";
	};
	dxvk-nvapi = pkgs.fetchurl {
		url = "https://github.com/jp7677/dxvk-nvapi/releases/download/v0.9.3/dxvk-nvapi-v0.9.3.tar.gz";
		hash = "sha256-KEAzw45GO6MdQEZtZQ0MQR329ku/jtPRnJygyt8x39M=";
	};
in
pkgs.runCommand "emperor-penguin-translation" { nativeBuildInputs = [ pkgs.zstd ]; } ''
	mkdir -p $out/system32 $out/syswow64 vkd3d-proton dxvk-nvapi
	tar --zstd -xf ${vkd3d-proton} -C vkd3d-proton --strip-components=1
	tar -xzf ${dxvk-nvapi} -C dxvk-nvapi
	cp ${pkgs.dxvk.bin}/x64/*.dll vkd3d-proton/x64/*.dll dxvk-nvapi/x64/*.dll $out/system32
	cp ${pkgs.dxvk.bin}/x32/*.dll vkd3d-proton/x86/*.dll dxvk-nvapi/x32/*.dll $out/syswow64
''
