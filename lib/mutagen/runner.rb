require_relative "config"
require_relative "coverage"
require_relative "worker_pool"
require_relative "reporter/console"
require_relative "reporter/json"

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

      puts "Testing #{mutations.length} mutations with #{@config.parallel_workers} workers..."
      puts ""

      # 4. Baseline check — ensure test suite passes without mutations
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

      # 5. Execute mutations
      pool = WorkerPool.new(
        workers: @config.parallel_workers,
        test_runner: test_runner,
        timeout_multiplier: @config["timeout_multiplier"]
      )

      results = pool.run(mutations)

      # 5. Report results
      duration = Process.clock_gettime(Process::CLOCK_MONOTONIC) - start_time
      score = Reporter::Console.new.report(results, total_duration: duration)

      # 6. Save JSON report
      Reporter::Json.new.report(results, output_path: "mutagen_results.json")
      puts "Results saved to mutagen_results.json"

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
        source = File.read(file)
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

      files.uniq.sort
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
