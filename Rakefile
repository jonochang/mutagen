require "rake/extensiontask"
require "rspec/core/rake_task"

Rake::ExtensionTask.new("mutagen_ruby") do |ext|
  ext.lib_dir = "lib/mutagen"
  ext.source_pattern = "*.{rs,toml}"
end

RSpec::Core::RakeTask.new(:spec)

task default: [:compile, :spec]
