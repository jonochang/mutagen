require "spec_helper"
require "mutagen/config"

RSpec.describe Mutagen::Config do
  it "has default settings" do
    config = Mutagen::Config.new
    expect(config["test_runner"]).to eq("rspec")
    expect(config["fail_under"]).to eq(80)
    expect(config["incremental"]).to be true
    expect(config["timeout_multiplier"]).to eq(3.0)
  end

  it "allows overrides" do
    config = Mutagen::Config.new("test_runner" => "minitest", "fail_under" => 90)
    expect(config["test_runner"]).to eq("minitest")
    expect(config["fail_under"]).to eq(90)
  end

  it "calculates parallel workers" do
    config = Mutagen::Config.new("parallel" => 4)
    expect(config.parallel_workers).to eq(4)
  end

  it "uses processor count for auto parallel" do
    config = Mutagen::Config.new("parallel" => "auto")
    expect(config.parallel_workers).to be > 0
  end
end
