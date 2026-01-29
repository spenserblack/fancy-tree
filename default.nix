let
  nixpkgs = import <nixpkgs> { };
in
  {
    rustPlatform ? nixpkgs.rustPlatform,
  }: rustPlatform.buildRustPackage rec {
    pname = "fancy-tree";
    version = "0.1.1";
    src = ./.;
    cargoLock = {
      lockFile = ./Cargo.lock;
    };
  }
