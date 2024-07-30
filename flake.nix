# For now, I have rustup in my path anyways, so this is more an
# illustration how to do this in general.
{
  description = "Striputary";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system:
  let
    overlays = [ (import rust-overlay) ];
    pkgs = import nixpkgs { inherit system overlays; };
    stable = pkgs.rust-bin.beta.latest.default.override {
      extensions = ["rust-src"];
    };
  in {
    devShells = with pkgs; {
      default = mkShell {
        packages = [
          stable
          # Bevy
          pkg-config
          clang
        ];
        buildInputs = [
          dbus
          fontconfig
          freetype
          alsa-lib
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          libxkbcommon
          libGL
          libGLU
        ];
        shellHook = ''
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.libGL}/lib";
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.libxkbcommon}/lib";
        '';
      };
    };
  });
}
