require "spec_helper"
require "mutagen/runner"

RSpec.describe Mutagen::Runner do
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
