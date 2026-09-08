{ pkgs ? import <nixpkgs> {
  overlays = [ (import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ];
} }:

let
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "llvm-tools-preview" ];
    targets = [ "thumbv8m.main-none-eabihf" ];
  };
in
pkgs.mkShell {
  buildInputs = [
    rustToolchain
    pkgs.rust-analyzer
    pkgs.picotool
  ];
}
