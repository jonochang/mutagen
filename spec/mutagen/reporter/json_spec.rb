require "spec_helper"
require "mutagen/worker_pool"
require "mutagen/reporter/json"
require "tempfile"
require "json"

RSpec.describe Mutagen::Reporter::Json do
  let(:reporter) { described_class.new }

  def make_result(file:, status:, operator: "arithmetic/+_to_-", original: "+", replacement: "-", line: 1, col: 5)
    Mutagen::WorkerPool::Result.new(
      mutation: { id: "test_id", file: file, line: line, col: col, operator: operator, original: original, replacement: replacement },
      status: status,
      killing_test: nil,
      duration_ms: 100
    )
  end

  describe "output routing" do
    it "writes to file when output_path is given" do
      results = [make_result(file: "app.rb", status: "killed")]

      tmpfile = Tempfile.new(["mutagen", ".json"])
      begin
        reporter.report(results, output_path: tmpfile.path)
        data = JSON.parse(File.read(tmpfile.path))
        expect(data["schemaVersion"]).to eq("1")
        expect(data["files"]).to have_key("app.rb")
      ensure
        tmpfile.unlink
      end
    end

    it "prints to stdout when output_path is nil" do
      results = [make_result(file: "app.rb", status: "killed")]

      output = nil
      expect { output = reporter.report(results) }.to output(/schemaVersion/).to_stdout
    end
  end

  describe "report structure" do
    it "includes schema version and thresholds" do
      results = [make_result(file: "app.rb", status: "killed")]

      tmpfile = Tempfile.new(["mutagen", ".json"])
      begin
        reporter.report(results, output_path: tmpfile.path)
        data = JSON.parse(File.read(tmpfile.path))

        expect(data["schemaVersion"]).to eq("1")
        expect(data["thresholds"]["high"]).to eq(80)
        expect(data["thresholds"]["low"]).to eq(60)
      ensure
        tmpfile.unlink
      end
    end

    it "groups mutants by file with correct fields" do
      results = [
        make_result(file: "app.rb", status: "killed", operator: "arithmetic/+_to_-", replacement: "-", line: 10, col: 5),
        make_result(file: "app.rb", status: "survived", operator: "boolean/true_to_false", replacement: "false", line: 20, col: 3),
      ]

      tmpfile = Tempfile.new(["mutagen", ".json"])
      begin
        reporter.report(results, output_path: tmpfile.path)
        data = JSON.parse(File.read(tmpfile.path))

        mutants = data["files"]["app.rb"]["mutants"]
        expect(mutants.length).to eq(2)

        killed = mutants.find { |m| m["status"] == "Killed" }
        expect(killed["mutatorName"]).to eq("arithmetic/+_to_-")
        expect(killed["replacement"]).to eq("-")
        expect(killed["location"]["start"]["line"]).to eq(10)
        expect(killed["location"]["start"]["column"]).to eq(5)

        survived = mutants.find { |m| m["status"] == "Survived" }
        expect(survived["mutatorName"]).to eq("boolean/true_to_false")
      ensure
        tmpfile.unlink
      end
    end
  end
end
