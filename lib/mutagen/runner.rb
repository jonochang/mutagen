require "json"
require_relative "config"
require_relative "coverage"
require_relative "worker_pool"
require_relative "reporter/console"
require_relative "reporter/json"
require_relative "reporter/html"

module Mutagen
  class Runner
    def initialize(config = nil)
      @config = config || Config.new
    end

    def run
      start_time = Process.clock_gettime(Process::CLOCK_MONOTONIC)

      # 1. Parse target files and generate mutations
      mutations = generate_mutations

      if mutations.empty?
        puts "No mutations generated."
        return 0
      end

      # 1b. Filter by enabled operators
      mutations = filter_by_operators(mutations)

      # 1c. Filter by inline ignore comments
      mutations = filter_by_ignore_comments(mutations)

      puts "Generated #{mutations.length} mutations across #{mutations.map { |m| m[:file] }.uniq.length} files"

      # 2. Filter by coverage
      if @config["coverage"]
        coverage_map = Coverage.load
        unless coverage_map.empty?
          mutations = filter_by_coverage(mutations, coverage_map)
          puts "After coverage filter: #{mutations.length} mutations"
        end
      end

      # 3. Apply sharding
      if @config["shard"]
        mutations = apply_shard(mutations)
        puts "After sharding: #{mutations.length} mutations"
      end

      # 4. Apply sampling
      mutations = apply_sampling(mutations)

      if mutations.empty?
        puts "No mutations to test after filtering."
        return 0
      end

      # 4b. Resume: load previous results and skip already-tested mutations
      previous_results = []
      if @config["resume"]
        previous_results, mutations = load_resume_data(mutations)
        if mutations.empty?
          puts "All #{previous_results.length} mutations already tested (resume mode)."
          results = previous_results
          duration = Process.clock_gettime(Process::CLOCK_MONOTONIC) - start_time
          score = Reporter::Console.new.report(results, total_duration: duration)
          Reporter::Json.new.report(results, output_path: "mutagen_results.json")
          Reporter::Html.new.report(results, output_path: "mutagen_report.html")
          puts "Results saved to mutagen_results.json and mutagen_report.html"
          return score < @config["fail_under"] ? 1 : 0
        end
        puts "Resume: #{previous_results.length} cached, #{mutations.length} remaining"
      end

      puts "Testing #{mutations.length} mutations with #{@config.parallel_workers} workers..."
      puts ""

      # 5. Baseline check — ensure test suite passes without mutations
      test_runner = build_test_runner
      baseline = test_runner.baseline_run
      unless baseline[:success]
        if baseline[:exit_code] == 127
          warn "ERROR: Test runner command not found. Is rspec/minitest installed?"
        else
          warn "ERROR: Baseline test suite failed (exit #{baseline[:exit_code]}). Fix your tests before running mutation testing."
        end
        return 1
      end

      # 6. Execute mutations
      pool = WorkerPool.new(
        workers: @config.parallel_workers,
        test_runner: test_runner,
        timeout_multiplier: @config["timeout_multiplier"]
      )

      results = previous_results + pool.run(mutations)

      # 5. Report results
      duration = Process.clock_gettime(Process::CLOCK_MONOTONIC) - start_time
      score = Reporter::Console.new.report(results, total_duration: duration)

      # 6. Save reports
      Reporter::Json.new.report(results, output_path: "mutagen_results.json")
      Reporter::Html.new.report(results, output_path: "mutagen_report.html")
      puts "Results saved to mutagen_results.json and mutagen_report.html"

      # 7. Check threshold
      threshold = @config["fail_under"]
      if score < threshold
        puts ""
        puts "FAIL: Mutation score #{score}% is below threshold #{threshold}%"
        return 1
      end

      0
    end

    private

    def generate_mutations
      target_files = find_target_files
      mutations = []

      target_files.each do |file|
        # Edge case: skip empty files
        next if File.zero?(file)

        # Edge case: skip very large files (>1MB)
        if File.size(file) > 1_048_576
          warn "Warning: skipping #{file} (>1MB)"
          next
        end

        source = File.read(file, encoding: "UTF-8")

        # Edge case: skip files with invalid encoding
        unless source.valid_encoding?
          warn "Warning: skipping #{file} (invalid UTF-8 encoding)"
          next
        end

        begin
          file_mutations = Mutagen::Native.generate_mutations(source, file)
          file_mutations.each do |m|
            m[:mutated_source] = Mutagen::Native.apply_mutation(
              source, m[:byte_range_start], m[:byte_range_end], m[:replacement]
            )
          end
          mutations.concat(file_mutations)
        rescue => e
          warn "Warning: failed to parse #{file}: #{e.message}"
        end
      end

      mutations
    end

    def find_target_files
      include_patterns = @config["include"]
      exclude_patterns = @config["exclude"]

      files = include_patterns.flat_map { |p| Dir.glob(p) }
      files.reject! do |f|
        exclude_patterns.any? { |p| File.fnmatch(p, f) }
      end

      # Filter by git diff if --since is set
      if @config["since"]
        ref = @config["since"]
        changed = `git diff --name-only #{ref} 2>/dev/null`.split("\n").map(&:strip)
        changed.concat(`git diff --name-only --cached #{ref} 2>/dev/null`.split("\n").map(&:strip))
        changed.uniq!
        files &= changed
        puts "After --since #{ref} filter: #{files.length} files"
      end

      files.uniq.sort
    end

    def filter_by_operators(mutations)
      mutators_config = @config["mutators"]
      return mutations unless mutators_config

      # --operator flag: run only one operator
      if mutators_config["one_op"]
        op = mutators_config["one_op"]
        return mutations.select { |m| m[:operator].start_with?(op) }
      end

      enabled = mutators_config["enabled"]
      disabled = mutators_config["disabled_operators"] || []

      mutations.select do |m|
        category = m[:operator].split("/").first
        enabled.include?(category) && !disabled.include?(category)
      end
    end

    def filter_by_ignore_comments(mutations)
      pattern = @config["ignore_pattern"]
      return mutations unless pattern

      # Cache file contents to avoid re-reading
      file_lines = {}

      mutations.reject do |m|
        file = m[:file]
        line = m[:line]
        next false if line == 0

        file_lines[file] ||= begin
          File.readlines(file)
        rescue
          []
        end

        lines = file_lines[file]
        line_text = lines[line - 1]
        line_text && line_text.include?(pattern)
      end
    end

    def filter_by_coverage(mutations, coverage_map)
      mutations.select do |m|
        lines = coverage_map[m[:file]]
        lines.nil? || lines.include?(m[:line])
      end
    end

    def apply_sampling(mutations)
      sampling = @config["sampling"].to_s
      if sampling.end_with?("%")
        percent = sampling.to_i
        return mutations if percent >= 100

        count = (mutations.length * percent / 100.0).round
        mutations.sample(count)
      elsif sampling =~ /^\d+$/
        count = sampling.to_i
        mutations.sample([count, mutations.length].min)
      else
        mutations
      end
    end

    def apply_shard(mutations)
      shard_spec = @config["shard"].to_s
      index, total = shard_spec.split("/").map(&:to_i)
      return mutations if total <= 1

      mutations.select do |m|
        m[:id].hash.abs % total == (index - 1)
      end
    end

    def load_resume_data(mutations)
      results_path = "mutagen_results.json"
      previous_results = []

      if File.exist?(results_path)
        data = JSON.parse(File.read(results_path))
        completed_ids = {}

        (data["files"] || {}).each do |file, file_data|
          (file_data["mutants"] || []).each do |mutant|
            completed_ids[mutant["id"]] = {
              status: mutant["status"].downcase,
              operator: mutant["mutatorName"],
              replacement: mutant["replacement"]
            }
          end
        end

        remaining = []
        mutations.each do |m|
          cached = completed_ids[m[:id]]
          if cached
            result = WorkerPool::Result.new(
              mutation: m,
              status: cached[:status],
              killing_test: nil,
              duration_ms: 0
            )
            previous_results << result
          else
            remaining << m
          end
        end

        [previous_results, remaining]
      else
        [[], mutations]
      end
    end

    def build_test_runner
      case @config["test_runner"]
      when "minitest"
        require_relative "test_runner/minitest"
        TestRunner::Minitest.new
      else
        require_relative "test_runner/rspec"
        TestRunner::RSpec.new
      end
    end
  end
end
