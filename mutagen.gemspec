require_relative "lib/mutagen/version"

Gem::Specification.new do |spec|
  spec.name = "mutagen"
  spec.version = Mutagen::VERSION
  spec.authors = ["Jonathan Chang"]
  spec.summary = "Mutation testing for Ruby, powered by Rust"
  spec.description = "A mutation testing framework with a Rust core for fast AST-level mutations and a Ruby orchestration layer."
  spec.homepage = "https://github.com/jonochang/mutagen"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.0"

  spec.files = Dir["lib/**/*.rb", "ext/**/*.{rs,toml}", "crates/**/*.{rs,toml}", "Cargo.toml", "LICENSE", "README.md"]
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/mutagen_ruby/Cargo.toml"]

  spec.add_dependency "rb_sys", "~> 0.9"
end
