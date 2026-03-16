# Mutagen - Architecture

## High-Level Overview

```
┌─────────────────────────────────────────────────────┐
│                   Ruby Layer                        │
│                                                     │
│  CLI / Rake Task                                    │
│       │                                             │
│       ▼                                             │
│  Mutagen::Runner  (orchestration)                   │
│       │                                             │
│       ├── Mutagen::Coverage   (SimpleCov adapter)   │
│       ├── Mutagen::TestRunner (RSpec / Minitest)    │
│       ├── Mutagen::Reporter   (HTML / JSON)         │
│       ├── Mutagen::Config     (YAML / CLI opts)     │
│       └── Mutagen::WorkerPool (fork-based parallel) │
│                │                                    │
└────────────────┼────────────────────────────────────┘
                 │  FFI via Magnus
┌────────────────┼────────────────────────────────────┐
│                ▼         Rust Core                   │
│                                                     │
│  mutagen_core (Rust library crate)                  │
│       │                                             │
│       ├── parser     (lib-ruby-parser → AST)        │
│       ├── mutators   (AST → mutated ASTs)           │
│       ├── codegen    (mutated AST → Ruby source)    │
│       ├── diff       (original ↔ mutated diff)      │
│       ├── selector   (coverage/diff filtering)      │
│       └── store      (incremental result cache)     │
│                                                     │
│  mutagen_ruby (Rust cdylib crate, Magnus bindings)  │
│       └── exposes mutagen_core to Ruby              │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Rust Core (`mutagen_core`)

A pure Rust library crate with no Ruby dependencies. This keeps it testable in isolation and potentially reusable outside Ruby.

### Modules

#### `parser`
- Wraps [lib-ruby-parser](https://github.com/lib-ruby-parser/lib-ruby-parser) to parse Ruby source into an AST.
- Preserves source locations (byte offsets, line/column) for every node so mutated code can be precisely spliced back into the original source.
- Exposes a `SourceFile` struct holding the original source bytes and the parsed AST.

#### `mutators`
- Each mutation operator is a struct implementing the `Mutator` trait:

```rust
pub struct Mutation {
    pub id: MutationId,
    pub file: PathBuf,
    pub line: u32,
    pub col: u32,
    pub operator: String,       // e.g. "arithmetic/plus_to_minus"
    pub original: String,       // original source fragment
    pub replacement: String,    // mutated source fragment
    pub byte_range: Range<usize>,
}

pub trait Mutator: Send + Sync {
    /// Return the operator category (e.g. "arithmetic").
    fn category(&self) -> &str;

    /// Return the operator name (e.g. "plus_to_minus").
    fn name(&self) -> &str;

    /// Visit the AST and return all possible mutations.
    fn generate(&self, source: &SourceFile) -> Vec<Mutation>;
}
```

- Built-in operator groups (v1):

| Category       | Examples                                                    |
|----------------|-------------------------------------------------------------|
| Arithmetic     | `+` → `-`, `*` → `/`, `%` → `*`                           |
| Comparison     | `>` → `>=`, `==` → `!=`, `<` → `>`                        |
| Boolean        | `&&` → `\|\|`, `!x` → `x`, `true` → `false`              |
| Conditional    | `if cond` → `if true` / `if false`, remove `else` branch  |
| Assignment     | `+=` → `-=`, `\|\|=` → `&&=`                              |
| Literal        | `0` → `1`, `""` → `"mutagen"`, `[]` → `[nil]`            |
| Return         | Remove `return expr`, `return expr` → `return nil`         |
| Statement      | Remove entire statement, swap adjacent statements           |
| Block          | Remove block body, `each` → `map` and vice versa           |
| Regex          | `\d` → `\w`, `+` → `*`, remove anchors                    |

- The `MutatorRegistry` holds all active mutators and filters them by config (enabled categories, excluded operators).

#### `codegen`
- Applies a `Mutation` to the original source bytes by replacing `byte_range` with the `replacement` string.
- For batch application (mutant schemata), wraps multiple mutations behind `if ENV["MUTAGEN_ID"] == "..."` guards in a single file. This is an optimisation for reducing file I/O — one file serves many mutants.

#### `selector`
- **Coverage filter**: Accepts a coverage map (`HashMap<PathBuf, HashSet<u32>>` — file → covered lines) and removes mutations on uncovered lines.
- **Diff filter**: Accepts a git diff (parsed from `git diff <ref>` output) and removes mutations outside changed hunks.
- **Sampling**: Applies random sampling (percentage or fixed count) to the remaining mutation set.
- Coverage data is produced by Ruby (SimpleCov) and passed into Rust as a serialized structure.

#### `store`
- Persists mutation results to a JSON file (`.mutagen_cache.json`) for incremental runs.
- Schema:

```json
{
  "version": 1,
  "runs": {
    "<mutation_id>": {
      "status": "killed" | "survived" | "timeout" | "error",
      "killing_test": "spec/models/user_spec.rb:42",
      "duration_ms": 120,
      "source_hash": "sha256 of original source file",
      "mutation_hash": "sha256 of mutation parameters"
    }
  }
}
```

- On subsequent runs, a mutation is skipped if `source_hash` and `mutation_hash` match and status is `killed` or `survived`.

## Rust → Ruby Bridge (`mutagen_ruby`)

A cdylib crate using [Magnus](https://github.com/matsadler/magnus) to expose Rust types to Ruby.

### Exposed Ruby API

```ruby
# All classes live under Mutagen::Native
module Mutagen
  module Native
    # Parse Ruby source and return mutations
    # source_code: String, config: Hash → Array<Hash>
    def self.generate_mutations(source_code, file_path, config = {})
      # Returns array of mutation hashes:
      # [{ id:, file:, line:, col:, operator:, original:, replacement:, byte_range: }]
    end

    # Apply a single mutation to source, return mutated source string
    def self.apply_mutation(source_code, mutation)
      # Returns String
    end

    # Apply mutant schemata (multiple mutations behind ENV guards)
    def self.apply_schemata(source_code, mutations)
      # Returns String
    end

    # Filter mutations by coverage data
    def self.filter_by_coverage(mutations, coverage_map)
      # Returns filtered Array<Hash>
    end

    # Filter mutations by git diff
    def self.filter_by_diff(mutations, diff_text)
      # Returns filtered Array<Hash>
    end

    # Load/save incremental cache
    def self.load_cache(path)
    def self.save_cache(path, results)
    def self.check_cache(cache, mutation)
      # Returns cached status or nil
    end
  end
end
```

### Data Marshalling

- Ruby Strings are passed as byte slices into Rust (zero-copy where possible via Magnus).
- Mutation structs are returned as Ruby Hashes (symbol keys) for easy consumption.
- Coverage maps are passed as `Hash<String, Array<Integer>>`.

## Ruby Layer

### `Mutagen::Config`
- Loads configuration from `.mutagen.yml` at the project root, merged with CLI arguments.

```yaml
# .mutagen.yml
mutators:
  enabled: [arithmetic, comparison, boolean, conditional]
  disabled_operators: [literal/empty_string]
  one_op: null                  # e.g. "arithmetic/plus_to_minus" for single-operator mode
parallel: auto                  # "auto" = number of CPU cores, or an integer
incremental: true
cache_path: .mutagen_cache.json
since: main                     # git ref for diff-based filtering
coverage: true                  # use SimpleCov data
sampling: 100%                  # or e.g. "25%" / "500"
schemata: false                 # mutant schemata batching (opt-in)
fail_under: 80                  # mutation score threshold for CI
timeout_multiplier: 3.0         # kill mutant if test takes 3x normal duration
test_runner: rspec              # or minitest
test_prioritisation: true       # order tests by kill history + speed
shard: null                     # e.g. "1/4" for CI sharding
include:
  - "app/**/*.rb"
  - "lib/**/*.rb"
exclude:
  - "app/views/**"
ignore_pattern: "mutagen:disable"
```

### `Mutagen::Runner`
Orchestrates the full mutation testing pipeline:

```
1. Load config
2. Run test suite once (baseline) → fail if tests don't pass
3. Collect coverage data (SimpleCov JSON)
4. For each target file:
   a. Call Native.generate_mutations(source, file, config)
   b. Filter by coverage, diff, sampling
   c. Check incremental cache → skip cached results
5. Distribute remaining mutations to WorkerPool
6. Collect results, update cache
7. Generate reports
8. Exit with appropriate code (pass/fail based on threshold)
```

### `Mutagen::WorkerPool`
- Forks `N` worker processes (one per CPU core by default).
- Each worker receives mutations from a queue (DRb or pipe-based IPC).
- Each worker:
  1. Sets `ENV["MUTAGEN_WORKER"] = worker_id` for resource isolation.
  2. For each mutation:
     a. Writes mutated source to a tempfile (or uses schemata).
     b. Requires the mutated file (overriding the original via `$LOAD_PATH` manipulation or `load`).
     c. Runs the relevant test(s) in a forked subprocess.
     d. Captures exit status and test output.
     e. Reports result (killed/survived/timeout/error) back to the main process.
- **Fail-fast per mutant**: Stop running tests for a mutant as soon as the first test fails (mutant killed).
- **Timeout**: Kill the test subprocess if it exceeds `timeout_multiplier * baseline_duration`.

### `Mutagen::TestRunner`
- Adapter interface for test frameworks.

```ruby
module Mutagen
  module TestRunner
    class Base
      def run_tests(test_files, env:) = raise NotImplementedError
      def baseline_run = raise NotImplementedError
    end

    class RSpec < Base
      # Shells out to: bundle exec rspec <files> --fail-fast --format json
    end

    class Minitest < Base
      # Shells out to: bundle exec ruby -e "ARGV.each { |f| require f }" <files>
    end
  end
end
```

### `Mutagen::Coverage`
- Reads SimpleCov's `.resultset.json` output.
- Converts it into the `Hash<String, Array<Integer>>` format that `Native.filter_by_coverage` expects.
- Maps test files to the source lines they cover, enabling per-mutant test selection.

### `Mutagen::Reporter`
- **JSON**: Full results dump conforming to mutation-testing-elements schema.
- **HTML**: Source-file view with inline annotations showing killed (green), survived (red), and skipped (grey) mutations per line.
- **Console**: Summary table with per-file mutation scores and overall score.

```
Mutation Score: 87.3% (1746/2000 killed)

File                           Mutations  Killed  Survived  Score
app/models/user.rb                    42      38         4  90.5%
app/services/payment.rb               67      52        15  77.6%
...

5 mutants timed out, 3 errored (excluded from score)
```

## Project Structure

```
mutagen/
├── Cargo.toml                  # Rust workspace
├── Gemfile
├── mutagen.gemspec
├── Rakefile                    # rake-compiler tasks for building native ext
├── ext/
│   └── mutagen_ruby/           # Rust cdylib crate (Magnus bindings)
│       ├── Cargo.toml
│       ├── build.rs
│       └── src/
│           └── lib.rs          # #[magnus::init] entry point
├── crates/
│   └── mutagen_core/           # Pure Rust library crate
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── parser.rs
│           ├── mutators/
│           │   ├── mod.rs
│           │   ├── arithmetic.rs
│           │   ├── comparison.rs
│           │   ├── boolean.rs
│           │   ├── conditional.rs
│           │   ├── literal.rs
│           │   ├── assignment.rs
│           │   ├── return_val.rs
│           │   ├── statement.rs
│           │   ├── block.rs
│           │   └── regex.rs
│           ├── codegen.rs
│           ├── selector.rs
│           ├── diff.rs
│           └── store.rs
├── lib/
│   └── mutagen/
│       ├── version.rb
│       ├── config.rb
│       ├── runner.rb
│       ├── worker_pool.rb
│       ├── coverage.rb
│       ├── reporter/
│       │   ├── json.rb
│       │   ├── html.rb
│       │   └── console.rb
│       ├── test_runner/
│       │   ├── base.rb
│       │   ├── rspec.rb
│       │   └── minitest.rb
│       └── native.rb           # require "mutagen/mutagen_ruby"
├── spec/                       # Ruby specs
├── tests/                      # Rust tests
└── docs/
    └── specs/
        ├── ideas.md
        ├── brief.md
        └── architecture.md
```

## Build & Distribution

1. **Development**: `bundle exec rake compile` uses `rb_sys` and `rake-compiler` to build the Rust extension into `lib/mutagen/mutagen_ruby.{so,dylib,bundle}`.
2. **CI**: GitHub Actions matrix builds precompiled gems for linux-x86_64, linux-aarch64, darwin-x86_64, darwin-arm64 using `rake-compiler-dock`.
3. **Installation**: `gem install mutagen` pulls the precompiled native gem for the user's platform. Falls back to source compilation if no precompiled binary matches.

## Data Flow

```
Ruby source files
       │
       ▼
  ┌─────────┐    lib-ruby-parser    ┌──────────┐    mutators     ┌────────────┐
  │  source  │ ──────────────────► │   AST    │ ─────────────► │  mutations │
  │  bytes   │                      │          │                 │   list     │
  └─────────┘                      └──────────┘                 └─────┬──────┘
                                                                      │
                   ┌──────────────────────────────────────────────────┘
                   │
    ┌──────────────┼───────────────┐
    │  filter by   │  filter by    │  check cache
    │  coverage    │  git diff     │
    └──────┬───────┴───────┬───────┘
           │               │
           ▼               ▼
    ┌─────────────────────────┐
    │  final mutation queue   │
    └────────────┬────────────┘
                 │
     ┌───────────┼───────────┐
     │           │           │
  Worker 1    Worker 2    Worker N
     │           │           │
  apply &     apply &     apply &
  run tests   run tests   run tests
     │           │           │
     └───────────┼───────────┘
                 │
                 ▼
          ┌────────────┐
          │  results   │ → cache update → reports
          └────────────┘
```

## Performance Strategies

This section consolidates all techniques used to reduce the cost of mutation testing, organised by the three pillars from the literature: **do fewer**, **do smarter**, and **do faster**.

### Do Fewer — Reduce the Number of Mutants and Test Executions

| Strategy | Where | How |
|---|---|---|
| **Selective mutation** | `Config`, `MutatorRegistry` | Users choose operator groups or individual operators. Default config enables a balanced subset; `--all-mutators` enables everything. |
| **One-op mode** | `Config`, CLI | `--operator arithmetic/plus_to_minus` runs a single operator for quick exploratory runs. |
| **Coverage-guided generation** | `selector` (Rust) | Mutations are only generated for lines covered by the test suite (via SimpleCov data). Uncovered code cannot produce killable mutants, so skipping it is free. |
| **Diff-based selection** | `selector` (Rust) | `--since <git-ref>` restricts mutations to lines changed since a reference commit. Ideal for CI on pull requests. |
| **Random sampling** | `selector` (Rust) | `sampling: 25%` or `--sample 500` randomly selects a subset of mutations. Maintains a representative mutation score while cutting volume. |
| **Trivial mutant avoidance** | `mutators` (Rust) | Mutator implementations embed rules to skip mutations that are syntactically equivalent or trivially detectable (e.g., mutating a constant that is immediately asserted in a test). Redundant mutants (e.g., `x > y` → `x >= y` when `x > y` → `x != y` already exists on the same node) are deduplicated. |
| **Test minimisation** | `Coverage` (Ruby) | For each mutation, only the tests that cover the mutated line are executed rather than the full suite. This is the single largest time saver for large projects. |

### Do Smarter — Leverage Reuse, Analysis, and Batching

| Strategy | Where | How |
|---|---|---|
| **Incremental caching** | `store` (Rust) | Results are persisted in `.mutagen_cache.json` keyed by source-file hash and mutation hash. Unchanged mutants are skipped on subsequent runs. `--resume` continues an interrupted session. |
| **Test prioritisation** | `WorkerPool` (Ruby) | Tests are ordered by (1) historical kill count (tests that killed mutants before are run first) and (2) duration (fastest tests first). This maximises the chance of killing a mutant on the first test, combining well with fail-fast. |
| **Mutant schemata (metamutants)** | `codegen` (Rust) | Multiple mutations in the same file are embedded behind `if ENV["MUTAGEN_ID"] == "..."` guards in a single compiled file. This eliminates repeated file I/O and, for projects with expensive `require`/`load` cycles, reduces startup cost per mutant. Opt-in via `schemata: true`. |
| **Equivalent mutant detection** | `mutators` (Rust) | Static analysis heuristics identify likely-equivalent mutants before execution. Examples: mutating a value that is immediately overwritten, negating a condition in dead code, replacing an operator in an expression whose result is unused. These are marked `skipped (equivalent)` in reports rather than counted as survived. |
| **Dead code elimination** | `selector` (Rust) | Basic control-flow analysis detects unreachable branches (e.g., code after an unconditional `return` or `raise`). Mutations in dead code are skipped. |
| **Sharding** | `Config`, CLI | `--shard 1/4` divides the mutation set deterministically into N equal shards. Each CI node runs one shard. Results are merged via `mutagen merge <result-files>` for a combined report. Shard assignment is stable (hash-based) so incremental caching remains effective across runs. |

### Do Faster — Optimise Execution Speed

| Strategy | Where | How |
|---|---|---|
| **Parallel workers** | `WorkerPool` (Ruby) | Fork-based worker pool defaults to one worker per CPU core. Each worker sets `ENV["MUTAGEN_WORKER"]` for resource isolation (e.g., unique database, port). |
| **Fail-fast per mutant** | `TestRunner` (Ruby) | Test execution for a mutant stops as soon as the first test fails (mutant is killed). No further tests are run for that mutant. |
| **Timeout detection** | `WorkerPool` (Ruby) | If a test run exceeds `timeout_multiplier * baseline_duration`, the subprocess is killed and the mutant is marked `timeout`. Prevents infinite loops from blocking the pipeline. |
| **Rust-native hot path** | `mutagen_core` (Rust) | Parsing, AST traversal, mutation generation, filtering, diffing, and caching are all performed in Rust. The Ruby ↔ Rust boundary is crossed only at batch boundaries (per-file), not per-node or per-mutation. |
| **Zero-copy data transfer** | `mutagen_ruby` (Magnus) | Source strings are passed from Ruby to Rust as byte slices without copying where possible. Mutation results are returned as lightweight Ruby hashes to minimise allocation overhead. |
| **Parallel file processing** | `mutagen_core` (Rust) | Multiple source files are parsed and mutated in parallel using Rayon within the Rust core, before results are returned to Ruby. This saturates CPU cores during the generation phase. |
| **Concurrency controls** | `Config`, CLI | `--jobs N` caps the number of parallel workers. For projects with resource-constrained test environments (shared database, limited ports), this prevents contention that would slow overall throughput. |

### Performance Configuration Summary

```yaml
# .mutagen.yml — performance-related options
parallel: auto                 # worker count ("auto" = CPU cores, or integer)
incremental: true              # reuse results from previous runs
cache_path: .mutagen_cache.json
since: main                    # diff-based filtering (git ref)
coverage: true                 # coverage-guided generation
sampling: 100%                 # "25%" or "500" for sampling
schemata: false                # mutant schemata batching (opt-in)
timeout_multiplier: 3.0        # timeout = multiplier * baseline duration
test_prioritisation: true      # order tests by kill history + speed
shard: null                    # e.g. "1/4" for CI sharding
mutators:
  enabled: [arithmetic, comparison, boolean, conditional]
  disabled_operators: []
  one_op: null                 # e.g. "arithmetic/plus_to_minus"
```

## Key Design Decisions

1. **Rust for parsing and mutation, Ruby for orchestration.** Parsing and AST traversal are CPU-bound and benefit from Rust. Test execution, process management, and reporting are I/O-bound and simpler in Ruby.

2. **lib-ruby-parser over tree-sitter.** lib-ruby-parser produces a full Ruby AST with precise source locations and is written in Rust natively. tree-sitter is grammar-based (CST) and would require more work to map to semantic mutation operators.

3. **Magnus over raw FFI.** Magnus provides safe, ergonomic Rust ↔ Ruby bindings with automatic type conversion and GC integration. It is the standard for modern Ruby native extensions in Rust.

4. **Fork-based isolation over in-process mutation.** Each mutant runs in a forked subprocess so a crash, infinite loop, or side effect cannot affect other mutants or the main process. The overhead of forking is acceptable given Ruby's copy-on-write friendly memory model.

5. **Byte-range replacement over AST rewriting.** Mutations are applied by replacing byte ranges in the original source string, not by serializing a modified AST. This preserves formatting, comments, and encoding exactly, and is simpler to implement.

6. **Mutant schemata as optional optimisation.** For large files with many mutations, embedding all mutations behind `ENV` guards in a single file avoids repeated file writes. This is opt-in and not the default path.
