require "spec_helper"
require "mutagen/worker_pool"
require "mutagen/reporter/console"

RSpec.describe Mutagen::Reporter::Console do
  let(:reporter) { described_class.new }

  def make_result(file:, status:, operator: "test", original: "x", replacement: "y", line: 1, col: 0)
    Mutagen::WorkerPool::Result.new(
      mutation: { id: "test", file: file, line: line, col: col, operator: operator, original: original, replacement: replacement },
      status: status,
      killing_test: nil,
      duration_ms: 100
    )
  end

  describe "score calculation" do
    it "calculates mutation score from killed and survived" do
      results = [
        make_result(file: "app.rb", status: "killed"),
        make_result(file: "app.rb", status: "killed"),
        make_result(file: "app.rb", status: "survived"),
      ]

      score = nil
      expect { score = reporter.report(results) }.to output(/66\.7%.*2 killed, 1 survived/m).to_stdout
      expect(score).to eq(66.7)
    end

    it "returns 100% when all mutations are killed" do
      results = [
        make_result(file: "app.rb", status: "killed"),
        make_result(file: "app.rb", status: "killed"),
      ]

      score = nil
      expect { score = reporter.report(results) }.to output(/100\.0%/).to_stdout
      expect(score).to eq(100.0)
    end

    it "returns 0% when all mutations survived" do
      results = [
        make_result(file: "app.rb", status: "survived"),
        make_result(file: "app.rb", status: "survived"),
      ]

      score = nil
      expect { score = reporter.report(results) }.to output(/0\.0%/).to_stdout
      expect(score).to eq(0.0)
    end

    it "returns 0% for empty results" do
      score = nil
      expect { score = reporter.report([]) }.to output(/0\.0%/).to_stdout
      expect(score).to eq(0.0)
    end
  end

  describe "timeout and error exclusion" do
    it "excludes timeouts from the score" do
      results = [
        make_result(file: "app.rb", status: "killed"),
        make_result(file: "app.rb", status: "survived"),
        make_result(file: "app.rb", status: "timeout"),
      ]

      score = nil
      expect { score = reporter.report(results) }.to output(/1 timed out.*excluded from score/m).to_stdout
      # Score should be 1 killed / (1 killed + 1 survived) = 50%
      expect(score).to eq(50.0)
    end

    it "excludes errors from the score" do
      results = [
        make_result(file: "app.rb", status: "killed"),
        make_result(file: "app.rb", status: "error"),
        make_result(file: "app.rb", status: "error"),
      ]

      score = nil
      expect { score = reporter.report(results) }.to output(/2 errored.*excluded from score/m).to_stdout
      # Score should be 1 killed / 1 scoreable = 100%
      expect(score).to eq(100.0)
    end

    it "shows both timeouts and errors when present" do
      results = [
        make_result(file: "app.rb", status: "killed"),
        make_result(file: "app.rb", status: "timeout"),
        make_result(file: "app.rb", status: "error"),
      ]

      expect { reporter.report(results) }.to output(/1 timed out, 1 errored.*excluded from score/m).to_stdout
    end
  end

  describe "per-file breakdown" do
    it "shows per-file scores" do
      results = [
        make_result(file: "a.rb", status: "killed"),
        make_result(file: "a.rb", status: "survived"),
        make_result(file: "b.rb", status: "killed"),
        make_result(file: "b.rb", status: "killed"),
      ]

      output = capture_output { reporter.report(results) }
      expect(output).to include("a.rb")
      expect(output).to include("b.rb")
      # a.rb: 1/2 = 50%, b.rb: 2/2 = 100%
      expect(output).to match(/a\.rb.*50\.0%/m)
      expect(output).to match(/b\.rb.*100\.0%/m)
    end
  end

  describe "survived mutations detail" do
    it "lists survived mutations with file, line, operator, and change" do
      results = [
        make_result(file: "app.rb", status: "survived", operator: "arithmetic/+_to_-", original: "+", replacement: "-", line: 10),
        make_result(file: "app.rb", status: "killed"),
      ]

      output = capture_output { reporter.report(results) }
      expect(output).to include("Survived mutations:")
      expect(output).to include("line 10: arithmetic/+_to_- — + -> -")
    end

    it "does not show survived section when all are killed" do
      results = [
        make_result(file: "app.rb", status: "killed"),
      ]

      output = capture_output { reporter.report(results) }
      expect(output).not_to include("Survived mutations:")
    end

    it "groups survived mutations by file" do
      results = [
        make_result(file: "a.rb", status: "survived", line: 1),
        make_result(file: "b.rb", status: "survived", line: 5),
      ]

      output = capture_output { reporter.report(results) }
      expect(output).to match(/a\.rb.*line 1.*b\.rb.*line 5/m)
    end
  end

  describe "duration" do
    it "shows completion time when total_duration is provided" do
      results = [make_result(file: "app.rb", status: "killed")]

      output = capture_output { reporter.report(results, total_duration: 5.123) }
      expect(output).to include("Completed in 5.1s")
    end

    it "does not show completion time when total_duration is nil" do
      results = [make_result(file: "app.rb", status: "killed")]

      output = capture_output { reporter.report(results) }
      expect(output).not_to include("Completed in")
    end
  end

  private

  def capture_output
    output = StringIO.new
    $stdout = output
    yield
    $stdout = STDOUT
    output.string
  end
end
