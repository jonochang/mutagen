# Changelog

## [0.3.0] - 2026-03-17

### Fixed

- Fix mutation execution: swap source files in-place with backup/restore instead of broken load path hack
- Detect command-not-found (exit 127/126) as errors instead of false "killed" results
- Add baseline check to fail early when test runner is missing or tests fail without mutations
- Fix line/col always being 0 in mutation reports — now computed from byte offsets

### Added

- Survived mutations detail in console report showing file, line, operator, and change for each surviving mutation
- JSON report automatically saved to `mutagen_results.json` after each run
- Per-file mutex locking for safe parallel mutation of shared source files
- File backup on disk (`.mutagen_backup`) for crash-safe restoration
- `LIBCLANG_PATH` and `BINDGEN_EXTRA_CLANG_ARGS` in nix dev shell for native extension builds
- 33 new RSpec specs (55 total): worker pool, console reporter, JSON reporter, config YAML loading, test runners, runner baseline/sampling/threshold

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
