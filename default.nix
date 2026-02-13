with import <nixpkgs> { };
callPackage ./packaging/nix/package.nix { inherit rustPlatform; }
