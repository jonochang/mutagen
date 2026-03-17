# Changelog

## [0.2.1] - 2026-03-17

### Fixed

- Fix nix package build on Linux: provide glibc headers to bindgen via `BINDGEN_EXTRA_CLANG_ARGS`

## [0.2.0] - 2026-03-17

### Added

- Native extension build support for macOS (`build.rs` with dynamic lookup linker flag)
- Magnus 0.8 API migration (fixes deprecation warnings)
- Rayon-based parallel batch processing for multi-file mutation generation
- Sharding support (`--shard N/TOTAL`) for distributing mutations across CI jobs
- Cache merge (`mutagen merge`) for combining sharded result files
- Sharding in Ruby runner with stable hash-based distribution
- 11 Ruby integration specs proving end-to-end Rust-to-Ruby native extension bridge
- 3 BDD scenarios for sharding (distribution, full coverage, single-shard passthrough)
- Comprehensive README with usage, configuration, CI integration, and development docs

### Changed

- Updated `ext/mutagen_ruby/src/lib.rs` to use `ruby.to_symbol()` instead of deprecated `Symbol::new()`
- Removed unused imports from native extension

## [0.1.0] - 2026-03-17

### Added

- Project scaffolding with Nix flake, Rust workspace, and Ruby gem skeleton
- Ruby parser via lib-ruby-parser v4 with manual AST walking
- 5 mutation operators: arithmetic, comparison, boolean, conditional, literal
- Byte-range replacement codegen
- Coverage-guided filtering via SimpleCov data
- Random sampling (by count or percentage)
- Incremental JSON cache with source hash validation
- Magnus 0.8 native extension bridge (`Mutagen::Native`)
- Fork-based worker pool with configurable parallelism
- RSpec and Minitest test runner adapters
- Console and JSON reporters
- CLI (`exe/mutagen`) with run, version, help commands
- 26 BDD scenarios for mutation operators
- 9 BDD scenarios for selector (coverage, sampling, cache)
- CI pipeline (GitHub Actions)
- cargo-deny, cargo-nextest configuration
