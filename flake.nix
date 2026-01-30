{
  description = "Displays file structure as a tree with Nerd Font icons, git statuses, etc.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }: {

    packages.x86_64-linux.fancy-tree =
      with import nixpkgs { system = "x86_64-linux"; };
      rustPlatform.buildRustPackage rec {
        pname = "fancy-tree";
        version = "0.1.2";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };

    packages.x86_64-linux.default = self.packages.x86_64-linux.fancy-tree;

  };
}
