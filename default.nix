{
  nixpkgs ? import <nixpkgs> {}
}:
nixpkgs.rustPlatform.buildRustPackage rec {
  name = "slidy-${version}";
  version = "0.0.1";

  buildInputs = with nixpkgs; [SDL2 SDL2_ttf SDL2_image];

  src = builtins.fetchGit ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  verifyCargoDeps = true;
}
