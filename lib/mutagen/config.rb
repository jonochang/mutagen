require "yaml"

module Mutagen
  class Config
    DEFAULTS = {
      "mutators" => {
        "enabled" => %w[arithmetic comparison boolean conditional literal assignment return_val statement block regex],
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
      @settings = deep_merge(DEFAULTS, deep_merge(load_file, overrides))
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

    def deep_merge(base, override)
      base.merge(override) do |_key, old_val, new_val|
        if old_val.is_a?(Hash) && new_val.is_a?(Hash)
          deep_merge(old_val, new_val)
        else
          new_val
        end
      end
    end

    def processor_count
      require "etc"
      Etc.nprocessors
    end
  end
end
