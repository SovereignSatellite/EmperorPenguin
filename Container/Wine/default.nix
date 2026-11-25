{ pkgs }:
let
	# GE-Proton11-12 patches Valve's Wine with its own script and the wine-staging commit it pins.
	ge = pkgs.fetchFromGitHub {
		owner = "GloriousEggroll";
		repo = "proton-ge-custom";
		rev = "GE-Proton11-12";
		hash = "sha256-ROk8/0NonJT2hH4ltv81vCH5LnKthTQfXGJUsVeEU2c=";
	};
	staging = pkgs.fetchFromGitHub {
		owner = "wine-staging";
		repo = "wine-staging";
		rev = "6cc805ea57132eeaf44764e9213823c9b8d0d300";
		hash = "sha256-Yhf0HjfrKxo/AY1jrK+iovwsx3hB/GZiWv2zeBGAels=";
	};
in
# Its 11.0 base matches nixpkgs' stable Wine, whose build, Mono and Gecko versions it expects.
pkgs.wineWow64Packages.stableFull.overrideAttrs (old: {
	pname = "proton-ge-wine";
	version = "11-12";
	src = pkgs.fetchFromGitHub {
		owner = "ValveSoftware";
		repo = "wine";
		rev = "fda79a585939fb7512dc220a11249f9ffe825fcf";
		hash = "sha256-MWH0QYcgo+Ld+xwn+24Fiz5J+P+1XF1cTyJB4JcBXLg=";
	};
	# GE's script reverts this commit with Git, which brings back the generated files that staging
	# patches. The fetched tree has no history to revert it from.
	patches = [
		(pkgs.fetchpatch {
			url = "https://github.com/ValveSoftware/wine/commit/e813ca5771658b00875924ab88d525322e50d39f.patch";
			revert = true;
			hash = "sha256-n0Hf3wQfh68eeQOp2qcHca/lmj2A7B3lVvKPFnx5m0c=";
		})
	];
	# As Proton builds it: Valve's tree lacks the check that links OpenCL on Linux, and GE's patches
	# leave Wine's tests unbuildable.
	configureFlags = old.configureFlags ++ [
		"--without-opencl"
		"--disable-tests"
	];
	# GE builds its VR bridge into Wine.
	buildInputs = old.buildInputs ++ [ pkgs.openxr-loader ];
	nativeBuildInputs = old.nativeBuildInputs ++ [
		pkgs.autoconf
		pkgs.git
		pkgs.perl
		pkgs.python3
	];
	# GE's script patches the tree from inside it, with its sources beside it. Its Git commands only
	# reset a checkout, and the revert above stands in for the one that doesn't. Valve's tree leaves
	# out the generated sources, and the Vulkan generator creates a cache folder it doesn't use.
	postPatch = ''
		cp -r ${staging} ../wine-staging
		cp -r ${ge}/wineopenxr ../wineopenxr
		chmod -R u+w ../wine-staging ../wineopenxr
		for patch in ${ge}/patches/wineopenxr/*.patch; do
			patch -d ../wineopenxr -Np1 < $patch
		done
		ln -s ${ge}/patches ../patches
		patchShebangs ../wine-staging autogen.sh tools dlls/winevulkan/make_vulkan
		pushd() { :; }
		popd() { :; }
		git() { :; }
		source <(sed -n '/^apply_patch()/,/^### (1)/p; /^### (2) WINE PATCHING/,/^### END WINE PATCHING/p' \
			${ge}/patches/protonprep-valve-staging.sh)
		unset -f pushd popd git
		patch -p1 < ${../../Resources/Wine/app-id-prefix.patch}
		patch -p1 < ${../../Resources/Wine/clipboard-thread.patch}
		XDG_CACHE_HOME=$TMPDIR ./autogen.sh
	'';
	# Valve's loader starts 64-bit processes from where Proton's packaging copies them.
	postInstall = old.postInstall + ''
		mkdir -p $out/lib/wine/i386-unix
		cp $out/lib/wine/x86_64-unix/{wine64,wine64-preloader} $out/lib/wine/i386-unix/
	'';
})
