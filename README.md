## Emperor Penguin

Emperor Penguin is a kind of opinionated, kind of specialized Wine-on-Podman container manager. It handles the more annoying parts of dealing with Wine prefixes and containerizing the programs you run under them.

### Why?

Windows is an affront to my existence, and it sucks that a lot of software will only run on it. Wine basically solves this problem, but it also demands a lot of boring know-how for basic things. I just want to click a button and have it work, and have configuration happen in neat little menus with tooltips and names. On top of that, I value my filesystem integrity a lot, and the thought of Windows or something running within it dropping a file anywhere within my pristine home directory makes me feel dirty at a spiritual level. For that reason, I also wanted basic containerization, intended to prevent "oopsies" rather than actively malicious behavior.

This project is built on my own needs and opinions, so it's going to be rough around the edges for just about anything that hasn't fit my use case. It "works on my machine", but I would love for it to work on as many machines as possible if anyone finds it useful.

This was tested on an `R9 9950X`, with an `RTX 5080`, and running `NixOS 26.11 (Zokor)`.

### Features

- Many boxes.
- Simple configuration window.
- One click run programs with pretty pictures.
- Podman containerization on:
  - Filesystem
  - Network
  - GPUs
  - Sound
  - Microphone
- Patched GE-Proton Wine because some of my games weren't playing right.
- DXVK, VKD3D-Proton, and DXVK-NVAPI work out of the box.
- `xdg-open` support for URI handling, both ways.
- Built on GPUI. (Not a feature, I just love Zed.)

### Installation

It's a Nix `flake` because that was convenient for me. It should build and "just work", but I'd like to support more platforms.

`nix run github:SovereignSatellite/EmperorPenguin`
