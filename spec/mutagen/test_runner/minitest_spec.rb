require "spec_helper"
require "mutagen/test_runner/minitest"

RSpec.describe Mutagen::TestRunner::Minitest do
  let(:runner) { described_class.new }

  describe "#baseline_run" do
    it "runs the command and returns a result hash" do
      result = runner.baseline_run

      expect(result).to have_key(:success)
      expect(result).to have_key(:exit_code)
      expect(result).to have_key(:stdout)
      expect(result).to have_key(:stderr)
    end

    it "sets baseline_duration after running" do
      expect(runner.baseline_duration).to be_nil
      runner.baseline_run
      expect(runner.baseline_duration).to be_a(Float)
      expect(runner.baseline_duration).to be > 0
    end
  end

  describe "#run_tests" do
    it "runs ruby with test files via ARGV" do
      result = runner.run_tests([])

      expect(result).to have_key(:success)
      expect(result).to have_key(:exit_code)
      # No test files = success (no-op)
      expect(result[:success]).to be true
    end

    it "reports failure for nonexistent test file" do
      result = runner.run_tests(["/nonexistent/test_file.rb"])

      expect(result[:success]).to be false
    end
  end
end
