require "spec_helper"
require "mutagen/config"
require "tempfile"

RSpec.describe Mutagen::Config do
  describe "defaults" do
    it "has default settings" do
      config = Mutagen::Config.new
      expect(config["test_runner"]).to eq("rspec")
      expect(config["fail_under"]).to eq(80)
      expect(config["incremental"]).to be true
      expect(config["timeout_multiplier"]).to eq(3.0)
    end

    it "includes all default mutator operators" do
      config = Mutagen::Config.new
      expect(config["mutators"]["enabled"]).to eq(%w[arithmetic comparison boolean conditional literal])
    end

    it "has default include patterns" do
      config = Mutagen::Config.new
      expect(config["include"]).to eq(["app/**/*.rb", "lib/**/*.rb"])
    end
  end

  describe "overrides" do
    it "allows overrides" do
      config = Mutagen::Config.new("test_runner" => "minitest", "fail_under" => 90)
      expect(config["test_runner"]).to eq("minitest")
      expect(config["fail_under"]).to eq(90)
    end

    it "preserves defaults for non-overridden keys" do
      config = Mutagen::Config.new("fail_under" => 50)
      expect(config["test_runner"]).to eq("rspec")
      expect(config["coverage"]).to be true
    end
  end

  describe "parallel_workers" do
    it "returns the numeric value when parallel is a number" do
      config = Mutagen::Config.new("parallel" => 4)
      expect(config.parallel_workers).to eq(4)
    end

    it "returns processor count when parallel is auto" do
      config = Mutagen::Config.new("parallel" => "auto")
      expect(config.parallel_workers).to be > 0
    end
  end

  describe "YAML file loading" do
    it "returns defaults when .mutagen.yml does not exist" do
      # Use a directory with no config file
      allow(Dir).to receive(:pwd).and_return("/nonexistent/path")
      config = Mutagen::Config.new
      expect(config["test_runner"]).to eq("rspec")
    end

    it "falls back to defaults when YAML file contains nil" do
      tmpdir = Dir.mktmpdir
      yaml_path = File.join(tmpdir, ".mutagen.yml")
      File.write(yaml_path, "---\n")  # YAML that parses to nil

      allow(Dir).to receive(:pwd).and_return(tmpdir)
      config = Mutagen::Config.new
      expect(config["test_runner"]).to eq("rspec")
    ensure
      FileUtils.rm_rf(tmpdir)
    end

    it "merges YAML config with defaults" do
      tmpdir = Dir.mktmpdir
      yaml_path = File.join(tmpdir, ".mutagen.yml")
      File.write(yaml_path, "test_runner: minitest\nfail_under: 95\n")

      allow(Dir).to receive(:pwd).and_return(tmpdir)
      config = Mutagen::Config.new
      expect(config["test_runner"]).to eq("minitest")
      expect(config["fail_under"]).to eq(95)
      # Defaults still present
      expect(config["coverage"]).to be true
    ensure
      FileUtils.rm_rf(tmpdir)
    end

    it "overrides take precedence over YAML file" do
      tmpdir = Dir.mktmpdir
      yaml_path = File.join(tmpdir, ".mutagen.yml")
      File.write(yaml_path, "test_runner: minitest\n")

      allow(Dir).to receive(:pwd).and_return(tmpdir)
      config = Mutagen::Config.new("test_runner" => "rspec")
      expect(config["test_runner"]).to eq("rspec")
    ensure
      FileUtils.rm_rf(tmpdir)
    end
  end
end
