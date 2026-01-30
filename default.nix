let
  nixpkgs = import <nixpkgs> { };
in
  {
    rustPlatform ? nixpkgs.rustPlatform,
    callPackage ? nixpkgs.callPackage,
  }: callPackage ./nix/package.nix { inherit rustPlatform; }
