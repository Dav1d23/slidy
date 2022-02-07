{
  nixpkgs ? import <nixpkgs> {}
}:
nixpkgs.rustPlatform.buildRustPackage rec {
  name = "slidy-${version}";
  version = "0.0.1";

  buildInputs = with nixpkgs; [SDL2 SDL2_ttf SDL2_image];

  src = builtins.fetchGit ./.;

  cargoSha256 = "1krdaabxmqm8xsyrjrkj4dyfs5a3043lk48a15gsdlm7amrpzczr";
  verifyCargoDeps = true;
}
