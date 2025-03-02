{ pkgs, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "xnode-demo";
  version = "1.0.0";
  src = ../rust-app;

  cargoLock = {
    lockFile = ../rust-app/Cargo.lock;
  };

  doDist = false;

  buildInputs = with pkgs; [
    openssl
  ];
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  meta = {
    mainProgram = "xnode-demo";
  };
}
