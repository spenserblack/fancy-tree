{ rustPlatform }: rustPlatform.buildRustPackage {
  pname = "fancy-tree";
  version = "0.1.4";
  src = ../../.;
  cargoLock = {
    lockFile = ../../Cargo.lock;
  };
}
