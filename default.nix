{
  nixpkgs ? import <nixpkgs> {}
}:
nixpkgs.rustPlatform.buildRustPackage rec {
  name = "slidy-${version}";
  version = "0.0.1";

  buildInputs = with nixpkgs; [SDL2 SDL2_ttf SDL2_image];

  src = builtins.fetchGit ./.;

  cargoSha256 = "0flk006wzqmqmg32qj6g56brhi64rs1vq64apdbgnldvn34rj6gr";
  verifyCargoDeps = true;
}
