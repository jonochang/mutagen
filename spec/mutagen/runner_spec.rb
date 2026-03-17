require "spec_helper"
require "mutagen/runner"
require "mutagen/test_runner/rspec"

RSpec.describe Mutagen::Runner do
  describe "no mutations" do
    it "returns 0 when no mutations are generated" do
      config = Mutagen::Config.new(
        "include" => ["nonexistent/**/*.rb"],
        "coverage" => false
      )
      runner = Mutagen::Runner.new(config)

      expect { @result = runner.run }.to output(/No mutations generated/).to_stdout
      expect(@result).to eq(0)
    end
  end

  describe "baseline check" do
    it "fails early when test runner command is not found" do
      config = Mutagen::Config.new(
        "include" => ["lib/**/*.rb"],
        "coverage" => false,
        "test_runner" => "rspec"
      )
      runner = Mutagen::Runner.new(config)

      # Stub baseline_run to simulate command not found
      fake_runner = instance_double(Mutagen::TestRunner::RSpec,
        baseline_duration: 10.0,
        baseline_run: { success: false, exit_code: 127, stdout: "", stderr: "" }
      )
      allow(runner).to receive(:build_test_runner).and_return(fake_runner)

      result = nil
      expect { result = runner.run }.to output(/ERROR.*not found/i).to_stderr
      expect(result).to eq(1)
    end

    it "fails early when baseline tests fail" do
      config = Mutagen::Config.new(
        "include" => ["lib/**/*.rb"],
        "coverage" => false
      )
      runner = Mutagen::Runner.new(config)

      fake_runner = instance_double(Mutagen::TestRunner::RSpec,
        baseline_duration: 10.0,
        baseline_run: { success: false, exit_code: 1, stdout: "", stderr: "" }
      )
      allow(runner).to receive(:build_test_runner).and_return(fake_runner)

      result = nil
      expect { result = runner.run }.to output(/ERROR.*Baseline.*failed/i).to_stderr
      expect(result).to eq(1)
    end
  end

  describe "sampling" do
    it "applies percentage-based sampling" do
      config = Mutagen::Config.new(
        "include" => ["lib/**/*.rb"],
        "coverage" => false,
        "sampling" => "10%"
      )
      runner = Mutagen::Runner.new(config)

      # The sampling reduces the number of mutations tested
      # We can't easily test the exact number, but we can verify it runs
      fake_runner = instance_double(Mutagen::TestRunner::RSpec,
        baseline_duration: 10.0,
        baseline_run: { success: true, exit_code: 0, stdout: "", stderr: "" }
      )
      allow(runner).to receive(:build_test_runner).and_return(fake_runner)

      # Stub WorkerPool to avoid real execution
      allow_any_instance_of(Mutagen::WorkerPool).to receive(:run).and_return([])

      expect { runner.run }.to output(/Testing \d+ mutations/m).to_stdout
    end
  end

  describe "threshold check" do
    it "returns 0 when score meets threshold" do
      config = Mutagen::Config.new(
        "include" => ["lib/**/*.rb"],
        "coverage" => false,
        "fail_under" => 0
      )
      runner = Mutagen::Runner.new(config)

      fake_runner = instance_double(Mutagen::TestRunner::RSpec,
        baseline_duration: 10.0,
        baseline_run: { success: true, exit_code: 0, stdout: "", stderr: "" }
      )
      allow(runner).to receive(:build_test_runner).and_return(fake_runner)
      allow_any_instance_of(Mutagen::WorkerPool).to receive(:run).and_return([])

      result = nil
      expect { result = runner.run }.to output.to_stdout
      expect(result).to eq(0)
    end
  end
end
