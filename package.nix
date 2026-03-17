{
  lib,
  stdenv,
  rustPlatform,
  ruby,
  makeWrapper,
  libclang,
  apple-sdk ? null,
}:

let
  version = "0.2.0";

  nativeExtension = rustPlatform.buildRustPackage {
    pname = "mutagen-native";
    inherit version;

    src = lib.cleanSourceWith {
      src = ./.;
      filter = path: type:
        let baseName = builtins.baseNameOf path; in
        (type == "directory") ||
        lib.hasSuffix ".rs" baseName ||
        lib.hasSuffix ".toml" baseName ||
        baseName == "Cargo.lock";
    };

    cargoHash = "sha256-ysG396tf1mw/3XTIfVz2W38BGE2w5nQbgZXTY4gOJvk=";

    nativeBuildInputs = [ ruby ];

    LIBCLANG_PATH = "${libclang.lib}/lib";

    # bindgen needs C standard library headers (stdio.h etc.)
    # On macOS, these come from the Apple SDK.
    BINDGEN_EXTRA_CLANG_ARGS = lib.optionalString stdenv.hostPlatform.isDarwin
      (let sdk = apple-sdk; in
        if sdk != null then
          "-isysroot ${sdk}/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
        else
          "");

    buildPhase = ''
      cargo build --release -p mutagen_ruby
    '';

    installPhase = let
      # Ruby loads .bundle on macOS, .so on Linux
      extSuffix = if stdenv.hostPlatform.isDarwin then "bundle" else "so";
    in ''
      mkdir -p $out/lib
      cp target/release/libmutagen_ruby${stdenv.hostPlatform.extensions.sharedLibrary} $out/lib/mutagen_ruby.${extSuffix}
    '';

    doCheck = false;
  };
in

stdenv.mkDerivation {
  pname = "mutagen";
  inherit version;

  src = lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      let baseName = builtins.baseNameOf path; in
      (type == "directory") ||
      lib.hasSuffix ".rb" baseName ||
      baseName == "mutagen";
  };

  nativeBuildInputs = [ makeWrapper ];
  buildInputs = [ ruby ];

  installPhase = ''
    mkdir -p $out/lib/mutagen $out/bin

    # Copy Ruby source
    cp -r lib/* $out/lib/

    # Copy native extension (.bundle on macOS, .so on Linux)
    cp ${nativeExtension}/lib/mutagen_ruby.* $out/lib/mutagen/

    # Copy CLI and wrap with correct RUBYLIB
    cp exe/mutagen $out/bin/mutagen
    chmod +x $out/bin/mutagen
    wrapProgram $out/bin/mutagen \
      --prefix RUBYLIB : "$out/lib" \
      --prefix PATH : "${ruby}/bin"
  '';

  meta = {
    description = "Mutation testing for Ruby, powered by Rust";
    homepage = "https://github.com/jonochang/mutagen";
    license = lib.licenses.mit;
    mainProgram = "mutagen";
    platforms = lib.platforms.unix;
  };
}
