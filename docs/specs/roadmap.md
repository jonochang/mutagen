# Mutagen - Roadmap

## Phase 0: Project Scaffolding & Toolchain

Set up the development environment, Rust workspace, Ruby gem skeleton, and CI pipeline before writing any application code.

### Nix Development Environment

Following the pattern from [untangle](../../../untangle):

- **`flake.nix`** with nixpkgs-unstable and rust-overlay:
  - Rust stable (latest) with extensions: `clippy`, `rustfmt`, `rust-src`, `llvm-tools-preview`
  - Ruby (>= 3.1) and Bundler
  - Native build deps: `pkg-config`, `openssl`, `libyaml`
  - Cargo dev tools: `cargo-nextest`, `cargo-deny`, `cargo-llvm-cov`, `cargo-insta`
  - `rb_sys` build dependencies for compiling native Ruby extensions
- **`.envrc`**: `use flake` for direnv integration
- **`default.nix`** / **`package.nix`**: Nix package definition for the gem's native extension

### Rust Workspace

```
Cargo.toml              # workspace root
├── crates/
│   └── mutagen_core/   # pure Rust library crate
│       └── Cargo.toml
└── ext/
    └── mutagen_ruby/   # cdylib crate (Magnus bindings)
        ├── Cargo.toml
        └── build.rs
```

- Workspace-level `[profile.release]` with `opt-level = "z"`, `lto = true`, `strip = true`, `codegen-units = 1`
- `mutagen_core`: zero Ruby dependencies, only `lib-ruby-parser`, `serde`, `serde_json`, `thiserror`, `rayon`
- `mutagen_ruby`: depends on `mutagen_core` + `magnus`, `rb_sys`

### Ruby Gem Skeleton

- `mutagen.gemspec` with `spec.extensions = ["ext/mutagen_ruby/Cargo.toml"]`
- `Gemfile` with dev deps: `rake`, `rake-compiler`, `rb_sys`, `rspec`, `simplecov`
- `Rakefile` with `rake-compiler` tasks for building the native extension
- `lib/mutagen.rb` entry point, `lib/mutagen/version.rb`

### Cargo Deny (`deny.toml`)

```toml
[graph]
all-features = true

[advisories]
version = 2

[licenses]
version = 2
allow = [
    "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause",
    "ISC", "MPL-2.0", "Unicode-3.0", "Unicode-DFS-2016",
    "Zlib", "BSL-1.0", "CC0-1.0",
]

[bans]
multiple-versions = "warn"
wildcards = "allow"

[sources]
unknown-registry = "warn"
unknown-git = "warn"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

### Cargo Nextest (`.config/nextest.toml`)

```toml
[profile.ci]
fail-fast = false
status-level = "fail"

[profile.ci.junit]
path = "target/nextest/ci/results.xml"
```

### Testing Setup

**Rust (mutagen_core):**
- Unit tests inline (`#[cfg(test)]` modules)
- Integration tests in `crates/mutagen_core/tests/`
- Snapshot tests via `cargo-insta` for parser output and codegen
- Property-based tests via `proptest` for mutator correctness

**Rust BDD (mutagen_core):**
- `cucumber` crate for feature specs
- `tests/features/` with `.feature` files for mutation generation scenarios
- Custom `World` struct that loads fixture Ruby files and asserts mutation output

**Ruby:**
- RSpec for the Ruby orchestration layer
- `spec/` directory with unit and integration specs
- BDD via RSpec feature specs or Cucumber-Ruby for end-to-end CLI testing

### CI Pipeline (`.github/workflows/ci.yml`)

```yaml
jobs:
  check:
    # cargo fmt --check
    # cargo clippy --all-targets -- -D warnings
    # cargo deny check

  test-rust:
    # cargo nextest run --profile ci
    # Upload JUnit results

  test-ruby:
    # bundle exec rake compile
    # bundle exec rspec

  build:
    # cargo build --release
    # bundle exec rake native gem
```

### Deliverables

- [ ] `flake.nix`, `.envrc`, `default.nix` — `nix develop` drops into a working shell
- [ ] Rust workspace compiles with empty `lib.rs` in both crates
- [ ] `bundle exec rake compile` builds the native extension and loads in Ruby
- [ ] `cargo nextest run` passes (trivial placeholder test)
- [ ] `bundle exec rspec` passes (trivial placeholder spec)
- [ ] `cargo deny check` passes
- [ ] `cargo clippy` and `cargo fmt --check` pass
- [ ] CI pipeline green on an empty scaffold commit

---

## Phase 1: Parser & First Mutator

Prove the end-to-end pipeline from Ruby source → AST → mutation → mutated source, crossing the Rust/Ruby boundary.

### Rust

- `parser` module: wrap `lib-ruby-parser`, expose `SourceFile` struct with AST and source bytes
- `Mutator` trait and `MutatorRegistry`
- First mutator: `arithmetic` (`+` → `-`, `-` → `+`, `*` → `/`, `/` → `*`)
- `codegen` module: byte-range replacement to produce mutated source string
- Snapshot tests (`cargo-insta`) for parser output and arithmetic mutations
- BDD features: `Given a Ruby file with "a + b"` / `When I generate mutations` / `Then a mutation replaces "+" with "-"`

### Ruby (Magnus bridge)

- `Mutagen::Native.generate_mutations(source, file_path, config)` → array of mutation hashes
- `Mutagen::Native.apply_mutation(source, mutation)` → mutated source string
- RSpec specs that call `Native.generate_mutations` on fixture files and verify output

### Deliverables

- [ ] `lib-ruby-parser` parses Ruby source and preserves source locations
- [ ] Arithmetic mutator generates correct mutations
- [ ] `apply_mutation` produces valid Ruby source
- [ ] Rust → Ruby bridge works end-to-end
- [ ] Snapshot and BDD tests cover the pipeline

---

## Phase 2: Core Mutator Suite

Implement the remaining v1 mutation operators.

### Mutators

- [ ] Comparison (`>` → `>=`, `==` → `!=`, etc.)
- [ ] Boolean (`&&` → `||`, `!x` → `x`, `true` → `false`)
- [ ] Conditional (`if cond` → `if true`/`if false`, remove `else`)
- [ ] Assignment (`+=` → `-=`, `||=` → `&&=`)
- [ ] Literal (`0` → `1`, `""` → `"mutagen"`, `[]` → `[nil]`)
- [ ] Return (remove `return`, `return expr` → `return nil`)
- [ ] Statement (remove statement, swap adjacent)
- [ ] Block (remove body, `each` ↔ `map`)
- [ ] Regex (`\d` → `\w`, `+` → `*`, remove anchors)

### Supporting work

- [ ] Trivial/equivalent mutant avoidance rules per mutator
- [ ] Redundant mutant deduplication in `MutatorRegistry`
- [ ] Config-driven operator selection (enabled groups, disabled operators)
- [ ] Inline ignore comments (`# mutagen:disable`)
- [ ] Comprehensive snapshot tests for each mutator
- [ ] Property tests: every mutation produces parseable Ruby

---

## Phase 3: Selector & Incremental Cache

Reduce the number of mutants that need execution.

### Rust

- [ ] `selector::coverage_filter` — filter by SimpleCov coverage map
- [ ] `selector::diff_filter` — parse unified diff, filter to changed hunks
- [ ] `selector::sample` — random sampling (percentage or fixed count)
- [ ] `store` — load/save `.mutagen_cache.json`, check cache hits by source + mutation hash
- [ ] Dead code elimination — skip mutations after unconditional `return`/`raise`

### Ruby (Magnus bridge)

- [ ] `Native.filter_by_coverage(mutations, coverage_map)`
- [ ] `Native.filter_by_diff(mutations, diff_text)`
- [ ] `Native.load_cache` / `Native.save_cache` / `Native.check_cache`
- [ ] `Mutagen::Coverage` — reads SimpleCov `.resultset.json`, converts to coverage map

### Deliverables

- [ ] Coverage filter removes mutations on uncovered lines
- [ ] Diff filter restricts to changed hunks for a given git ref
- [ ] Cache skips unchanged mutants on repeat runs
- [ ] `--resume` continues an interrupted session

---

## Phase 4: Test Runner & Worker Pool

Execute mutants against the test suite.

### Ruby

- [ ] `Mutagen::TestRunner::Base` adapter interface
- [ ] `Mutagen::TestRunner::RSpec` — shells out with `--fail-fast --format json`
- [ ] `Mutagen::TestRunner::Minitest` — shells out to run selected test files
- [ ] Baseline run: execute full suite, record per-test duration, fail if tests don't pass
- [ ] `Mutagen::WorkerPool` — fork N workers, pipe-based IPC, mutation queue
- [ ] Per-worker `ENV["MUTAGEN_WORKER"]` isolation
- [ ] Fail-fast per mutant (stop on first killing test)
- [ ] Timeout detection (`timeout_multiplier * baseline_duration`)
- [ ] Test prioritisation: order by historical kill count, then by duration (fastest first)
- [ ] Per-mutant test selection: only run tests covering the mutated lines

### Deliverables

- [ ] Single-worker mode works end-to-end: mutate → run tests → report killed/survived
- [ ] Parallel workers produce identical results to single-worker
- [ ] Timeouts and errors are handled gracefully
- [ ] Test prioritisation measurably reduces time-to-kill

---

## Phase 5: CLI, Config & Reporting

User-facing interface and output.

### CLI (`exe/mutagen`)

- [ ] `mutagen run` — full mutation testing pipeline
- [ ] `mutagen run --since main` — diff-based
- [ ] `mutagen run --sample 25%` — random sampling
- [ ] `mutagen run --shard 1/4` — CI sharding
- [ ] `mutagen run --operator arithmetic` — single-category or single-operator mode
- [ ] `mutagen merge <result-files>` — merge sharded results
- [ ] `mutagen report <cache-file>` — regenerate reports from cache

### Config (`.mutagen.yml`)

- [ ] `Mutagen::Config` — load YAML, merge with CLI args
- [ ] All options from architecture spec: mutators, parallel, incremental, coverage, sampling, schemata, thresholds, include/exclude, ignore pattern

### Reporting

- [ ] `Mutagen::Reporter::Console` — summary table with per-file scores
- [ ] `Mutagen::Reporter::JSON` — full results dump (mutation-testing-elements schema)
- [ ] `Mutagen::Reporter::HTML` — source overlay with per-line annotations
- [ ] CI exit code: non-zero if mutation score < `fail_under` threshold

### Deliverables

- [ ] `mutagen run` works end-to-end on a real Ruby project
- [ ] All CLI flags documented in `--help`
- [ ] HTML report renders correctly
- [ ] CI threshold enforcement works

---

## Phase 6: Performance & Polish

Optimisation pass and production hardening.

- [ ] Mutant schemata (`codegen::apply_schemata`) — opt-in via `schemata: true`
- [ ] Parallel file processing in Rust via Rayon
- [ ] Sharding with stable hash-based assignment
- [ ] `mutagen merge` for combining shard results
- [ ] Benchmark suite: measure generation speed, execution throughput, memory usage
- [ ] Profile and optimise the Rust hot path (parsing + mutation generation)
- [ ] Precompiled gem builds via `rake-compiler-dock` (linux-x86_64, linux-aarch64, darwin-x86_64, darwin-arm64)
- [ ] Edge case hardening: encoding issues, syntax errors in source, empty files, massive files

---

## Phase 7: Distribution & Documentation

Ship it.

- [ ] `gem push` to RubyGems with precompiled native binaries
- [ ] README with quickstart, configuration reference, CI integration guide
- [ ] `docs/` site via mdBook or similar, deployed to GitHub Pages
- [ ] CHANGELOG
- [ ] GitHub Actions release workflow: tag → test → build native gems → publish

---

## Future (v2+)

Deferred from v1 scope:

- Higher-order mutation (combine multiple mutations per mutant)
- Weak/firm mutation (early state-infection detection)
- Automatic test generation to kill surviving mutants
- Equivalent mutant detection via dynamic analysis (run both original and mutant, compare output)
- LSP/IDE integration (show surviving mutants inline)
- Plugin API for custom mutators defined in Ruby
- Rails-specific mutators (route removal, migration rollback, callback removal)
