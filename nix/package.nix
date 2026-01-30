{ rustPlatform }: rustPlatform.buildRustPackage {
  pname = "fancy-tree";
  version = "0.1.2";
  src = ../.;
  cargoLock = {
    lockFile = ../Cargo.lock;
  };
}
