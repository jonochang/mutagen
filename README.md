# Mutagen

A mutation testing framework for Ruby, powered by a Rust core for fast AST-level mutations. Mutagen modifies your source code in small ways (mutations) and runs your tests against each change to find weaknesses in your test suite.

## Features

- **Rust-powered mutation engine** -- parses Ruby via [lib-ruby-parser](https://github.com/lib-ruby-parser/lib-ruby-parser), generates mutations at the AST level, and applies them via byte-range replacement
- **5 mutation operators** -- arithmetic, comparison, boolean, conditional, literal
- **Coverage-guided filtering** -- skip mutations on lines not covered by your tests (via SimpleCov)
- **Incremental caching** -- skip unchanged mutations on repeat runs
- **Parallel execution** -- fork-based worker pool with configurable concurrency
- **CI sharding** -- split mutations across CI jobs with `--shard N/TOTAL`
- **Flexible test runners** -- RSpec and Minitest adapters

## Requirements

- Ruby >= 3.0
- Rust toolchain (for building the native extension)

## Installation

Add to your Gemfile:

```ruby
gem "mutagen"
```

Then build the native extension:

```sh
bundle exec rake compile
```

## Usage

### Basic run

```sh
mutagen run
```

This will:
1. Find Ruby files matching your configured include patterns
2. Generate mutations using all enabled operators
3. Filter by coverage data (if available)
4. Execute each mutation against your test suite in parallel
5. Report which mutations survived (indicating test gaps)

### CLI options

```
mutagen run [options]

  --since REF          Only mutate code changed since git ref
  --sample AMOUNT      Sample mutations (e.g. "25%" or "500")
  --shard N/TOTAL      Run shard N of TOTAL (e.g. "1/4")
  --operator OP        Run single operator (e.g. "arithmetic")
  --jobs, -j N         Number of parallel workers
  --fail-under N       Minimum mutation score (default: 80)
  --test-runner NAME   Test runner: rspec or minitest
  --no-coverage        Disable coverage-based filtering
  --no-incremental     Disable incremental caching
```

### Merge sharded results

After running sharded mutations across CI jobs:

```sh
mutagen merge shard1.json shard2.json shard3.json shard4.json
```

### Other commands

```sh
mutagen version    # Show version
mutagen help       # Show help
```

## Configuration

Create a `.mutagen.yml` in your project root:

```yaml
# Files to mutate
include:
  - app/**/*.rb
  - lib/**/*.rb
exclude: []

# Mutation operators
mutators:
  enabled:
    - arithmetic
    - comparison
    - boolean
    - conditional
    - literal

# Execution
parallel: auto          # Number of workers ("auto" uses all cores)
test_runner: rspec      # rspec or minitest
timeout_multiplier: 3.0

# Filtering
coverage: true          # Use SimpleCov data to skip uncovered lines
incremental: true       # Cache results between runs
cache_path: .mutagen_cache.json
sampling: "100%"        # e.g. "25%" or "500"

# Thresholds
fail_under: 80          # Exit non-zero if mutation score is below this
```

## Mutation operators

| Category | Mutations |
|---|---|
| **Arithmetic** | `+` <-> `-`, `*` <-> `/`, `%` -> `*` |
| **Comparison** | `>` <-> `>=`, `<` <-> `<=`, `>` <-> `<`, `>=` <-> `<=`, `==` <-> `!=` |
| **Boolean** | `&&` <-> `\|\|`, `true` <-> `false` |
| **Conditional** | `if cond` -> `if true` / `if false`, remove `else` branch |
| **Literal** | `0` <-> `1`, `N` -> `0`, `""` <-> `"mutagen"`, `[]` -> `[nil]` |

## CI integration

### GitHub Actions

```yaml
jobs:
  mutation-test:
    strategy:
      matrix:
        shard: [1, 2, 3, 4]
    steps:
      - uses: actions/checkout@v4
      - uses: ruby/setup-ruby@v1
        with:
          ruby-version: "3.3"
      - run: bundle install
      - run: bundle exec rake compile
      - run: mutagen run --shard ${{ matrix.shard }}/4 --fail-under 80
```

## Development

### Prerequisites

With [Nix](https://nixos.org/):

```sh
nix develop   # Drops into a shell with Rust, Ruby, and all dev tools
```

Or manually install Rust and Ruby >= 3.0.

### Running tests

```sh
# Rust tests (unit + BDD)
cargo test -p mutagen_core

# Ruby tests
cargo build -p mutagen_ruby
cp target/debug/libmutagen_ruby.dylib lib/mutagen/mutagen_ruby.bundle
bundle exec rspec
```

### Project structure

```
Cargo.toml                    # Workspace root
crates/mutagen_core/          # Pure Rust library (parser, mutators, codegen)
ext/mutagen_ruby/             # Ruby native extension (Magnus bindings)
lib/mutagen/                  # Ruby orchestration layer
exe/mutagen                   # CLI entry point
tests/features/               # BDD feature files
spec/                         # RSpec tests
```

## License

MIT
