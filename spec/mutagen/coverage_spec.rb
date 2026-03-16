require "spec_helper"
require "mutagen/coverage"
require "tempfile"
require "json"

RSpec.describe Mutagen::Coverage do
  it "loads SimpleCov resultset" do
    data = {
      "RSpec" => {
        "coverage" => {
          "/app/models/user.rb" => {
            "lines" => [1, 1, nil, 0, 1]
          }
        }
      }
    }

    tmpfile = Tempfile.new([".resultset", ".json"])
    tmpfile.write(JSON.generate(data))
    tmpfile.close

    coverage = Mutagen::Coverage.load(tmpfile.path)

    expect(coverage["/app/models/user.rb"]).to contain_exactly(1, 2, 5)

    tmpfile.unlink
  end

  it "returns empty hash for missing file" do
    coverage = Mutagen::Coverage.load("/nonexistent/path.json")
    expect(coverage).to eq({})
  end

  it "handles array-style coverage data" do
    data = {
      "RSpec" => {
        "coverage" => {
          "/app/models/user.rb" => [1, 0, nil, 1]
        }
      }
    }

    tmpfile = Tempfile.new([".resultset", ".json"])
    tmpfile.write(JSON.generate(data))
    tmpfile.close

    coverage = Mutagen::Coverage.load(tmpfile.path)

    expect(coverage["/app/models/user.rb"]).to contain_exactly(1, 4)

    tmpfile.unlink
  end
end
