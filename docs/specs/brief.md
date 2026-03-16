# Mutagen - Project Brief

## Overview

Mutagen is a mutation testing framework for Ruby projects. The core mutation engine is written in Rust for performance, exposed to Ruby as a native extension gem via [Magnus](https://github.com/matsadler/magnus) (Rust ↔ Ruby bindings). Ruby handles orchestration, test runner integration, and reporting.

## Problem

Mutation testing is the gold standard for measuring test suite quality, but existing Ruby tools (e.g. mutant) are slow due to Ruby's interpreted nature. Parsing ASTs, generating mutants, diffing source, and managing large mutant sets are CPU-bound tasks that benefit enormously from native code.

## Goals

1. **Fast mutation generation** - Rust parses Ruby source into ASTs, applies mutation operators, and produces mutated source files. This is the hot path and the primary reason for the Rust core.
2. **Practical by default** - Coverage-guided generation, diff-based selection (`--since`), incremental result caching, and fail-fast execution make mutation testing viable in CI and local development.
3. **Parallel execution** - Fork-based worker pool (controlled from Ruby) runs mutants across CPU cores. Support sharding for distribution across machines.
4. **Extensible** - Plugin API for custom mutation operators (defined in Ruby or Rust). Configurable operator groups, include/exclude patterns, and inline ignore comments.
5. **Actionable reporting** - HTML and JSON reports overlaying mutation results on source. CI-friendly threshold checks and machine-readable output.

## Non-Goals (v1)

- Higher-order mutation (combining multiple mutations per mutant).
- Weak/firm mutation (early state-infection detection) - complex to implement in Ruby's runtime; defer to v2.
- Automatic test generation to kill surviving mutants.
- Support for languages other than Ruby.

## Target Users

Ruby developers and teams who use RSpec or Minitest and want to measure and improve their test suite quality, especially in CI pipelines.

## Key Constraints

- **Ruby compatibility**: MRI Ruby >= 3.1. No JRuby/TruffleRuby support initially (native extension).
- **Rust toolchain**: Requires Rust stable for building the native extension. The gem will ship precompiled binaries for common platforms (linux-x86_64, linux-aarch64, darwin-x86_64, darwin-arm64) via `rb_sys` / `rake-compiler-dock`.
- **Test runners**: RSpec and Minitest support in v1.
- **Ruby parser**: The Rust core uses [lib-ruby-parser](https://github.com/lib-ruby-parser/lib-ruby-parser) to parse Ruby source into ASTs within Rust, avoiding round-trips to Ruby for parsing.

## Success Criteria

- Mutation generation is at least 10x faster than a pure-Ruby equivalent for projects with 1000+ mutation points.
- A full mutation run on a medium Rails app (~500 files, ~5000 tests) completes in under 10 minutes on an 8-core machine with coverage-guided + diff-based selection.
- Mutation score results are consistent and reproducible across runs.
