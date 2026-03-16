require "spec_helper"
require "mutagen/worker_pool"
require "mutagen/reporter/console"

RSpec.describe Mutagen::Reporter::Console do
  let(:reporter) { described_class.new }

  def make_result(file:, status:)
    Mutagen::WorkerPool::Result.new(
      mutation: { id: "test", file: file, line: 1, col: 0, operator: "test" },
      status: status,
      killing_test: nil,
      duration_ms: 100
    )
  end

  it "calculates mutation score" do
    results = [
      make_result(file: "app.rb", status: "killed"),
      make_result(file: "app.rb", status: "killed"),
      make_result(file: "app.rb", status: "survived"),
    ]

    score = nil
    expect { score = reporter.report(results) }.to output(/66\.7%/).to_stdout
    expect(score).to eq(66.7)
  end

  it "handles empty results" do
    score = nil
    expect { score = reporter.report([]) }.to output(/0%/).to_stdout
    expect(score).to eq(0.0)
  end
end
