{
  nixpkgs ? import <nixpkgs> {}
}:
nixpkgs.rustPlatform.buildRustPackage rec {
  name = "slidy-${version}";
  version = "0.0.1";

  buildInputs = with nixpkgs; [SDL2 SDL2_ttf SDL2_image];

  src = builtins.fetchGit ./.;

  cargoSha256 = "05kz4zlh40kmdll9zf1nz82pmd6nsizp5inacda9rxbnml800dqw";
  verifyCargoDeps = true;
}
