# This file goes in nixpkgs at: pkgs/by-name/mu/mutagen/package.nix
#
# To get the real cargoHash, set it to lib.fakeHash, run `nix-build -A mutagen`,
# and the error output will contain the correct hash.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  ruby,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "mutagen";
  version = "0.2.0";

  src = fetchFromGitHub {
    owner = "jonochang";
    repo = "mutagen";
    rev = "v${finalAttrs.version}";
    hash = lib.fakeHash;
  };

  cargoHash = lib.fakeHash;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    ruby
  ];

  # Only test the pure Rust core crate (mutagen_ruby needs Ruby runtime)
  cargoTestFlags = [
    "-p" "mutagen_core"
  ];

  meta = {
    description = "Mutation testing for Ruby, powered by Rust";
    homepage = "https://github.com/jonochang/mutagen";
    changelog = "https://github.com/jonochang/mutagen/blob/v${finalAttrs.version}/CHANGELOG.md";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ jonochang ];
    platforms = lib.platforms.unix;
  };
})
