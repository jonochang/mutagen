fn main() {
    // On macOS, Ruby native extensions (.bundle) resolve Ruby C API symbols
    // at load time, not link time. We need to tell the linker to allow
    // undefined symbols that will be provided by the Ruby runtime.
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-undefined,dynamic_lookup");
    }
}
