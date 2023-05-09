{
  description = "A simple flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default];
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = let
            arm-gcc =
              if system == "aarch64-darwin"
              then (pkgs // {system = "x86_64-darwin";}).gcc-arm-embedded
              else pkgs.gcc-arm-embedded;
            rust-toolchain = fenix.packages.${system}.combine (with fenix.packages.${system}; [
              latest.toolchain
              targets.thumbv7em-none-eabihf.latest.rust-std
            ]);
          in
            with pkgs; [
              arm-gcc
              cargo-binutils
              cargo-embed
              cargo-flash
              cargo-make
              dfu-util
              probe-run
              rust-toolchain
              stlink-gui
            ];
        };
      }
    );
}
