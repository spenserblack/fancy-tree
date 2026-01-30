{
  description = "Displays file structure as a tree with Nerd Font icons, git statuses, etc.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }: {

    packages.x86_64-linux.fancy-tree =
      with import nixpkgs { system = "x86_64-linux"; };
      callPackage ./nix/package.nix { inherit rustPlatform; };

    packages.x86_64-linux.default = self.packages.x86_64-linux.fancy-tree;

  };
}
