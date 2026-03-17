require "spec_helper"
require "mutagen/test_runner/rspec"

RSpec.describe Mutagen::TestRunner::RSpec do
  let(:runner) { described_class.new }

  describe "#baseline_run" do
    it "runs bundle exec rspec and returns a result hash" do
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

    it "reports success when tests pass" do
      result = runner.baseline_run
      expect(result[:success]).to be true
      expect(result[:exit_code]).to eq(0)
    end
  end

  describe "#run_tests" do
    it "runs rspec with specified test files" do
      # Run with a non-existent file to get a predictable failure
      result = runner.run_tests(["spec/nonexistent_spec.rb"])

      expect(result).to have_key(:success)
      expect(result).to have_key(:exit_code)
      # rspec exits 1 when it can't find the file
      expect(result[:success]).to be false
    end

    it "passes environment variables" do
      result = runner.run_tests(["spec/mutagen/version_spec.rb"], env: { "MUTAGEN_TEST" => "1" })

      expect(result).to have_key(:success)
      expect(result[:exit_code]).to be_a(Integer)
    end
  end
end
