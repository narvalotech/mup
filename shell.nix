{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rust-analyzer

    pkgs.picotool # needs extra udev rules
  ];
}
