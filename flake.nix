{
  description = "Displays file structure as a tree with Nerd Font icons, git statuses, etc.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: 
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    packages.${system} = rec {
      fancy-tree = pkgs.callPackage ./nix/package.nix { };
      default = fancy-tree;
    };
  };
}
