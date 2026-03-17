{
  description = "mutagen - mutation testing for Ruby, powered by Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "clippy" "rustfmt" "rust-src" "llvm-tools-preview" ];
        };

        ruby = pkgs.ruby_3_3;
      in
      {
        packages.default = pkgs.callPackage ./package.nix { inherit ruby; };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain

            # Ruby
            ruby
            pkgs.bundler

            # Native build dependencies
            pkgs.pkg-config
            pkgs.openssl
            pkgs.libyaml
            pkgs.libclang.lib

            # Cargo dev tools
            pkgs.cargo-nextest
            pkgs.cargo-deny
            pkgs.cargo-llvm-cov
            pkgs.cargo-insta

            # Documentation
            pkgs.mdbook
          ];

          env = {
            OPENSSL_DIR = "${pkgs.openssl.dev}";
            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS =
              if pkgs.stdenv.hostPlatform.isDarwin then
                "-isysroot ${pkgs.apple-sdk or ""}/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
              else
                "-isystem ${pkgs.stdenv.cc.libc.dev}/include";
          };
        };
      }
    );
}
