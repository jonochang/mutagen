require "yaml"

module Mutagen
  class Config
    DEFAULTS = {
      "mutators" => {
        "enabled" => %w[arithmetic comparison boolean conditional literal],
        "disabled_operators" => [],
        "one_op" => nil
      },
      "parallel" => "auto",
      "incremental" => true,
      "cache_path" => ".mutagen_cache.json",
      "since" => nil,
      "coverage" => true,
      "sampling" => "100%",
      "schemata" => false,
      "fail_under" => 80,
      "timeout_multiplier" => 3.0,
      "test_runner" => "rspec",
      "test_prioritisation" => true,
      "shard" => nil,
      "include" => ["app/**/*.rb", "lib/**/*.rb"],
      "exclude" => [],
      "ignore_pattern" => "mutagen:disable"
    }.freeze

    attr_reader :settings

    def initialize(overrides = {})
      @settings = DEFAULTS.merge(load_file).merge(overrides)
    end

    def [](key)
      @settings[key]
    end

    def parallel_workers
      val = @settings["parallel"]
      val == "auto" ? processor_count : val.to_i
    end

    private

    def load_file
      path = File.join(Dir.pwd, ".mutagen.yml")
      return {} unless File.exist?(path)

      YAML.safe_load(File.read(path)) || {}
    end

    def processor_count
      require "etc"
      Etc.nprocessors
    end
  end
end
