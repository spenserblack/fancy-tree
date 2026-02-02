{ rustPlatform }: rustPlatform.buildRustPackage {
  pname = "fancy-tree";
  version = "0.1.3";
  src = ../.;
  cargoLock = {
    lockFile = ../Cargo.lock;
  };
}
