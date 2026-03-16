require_relative "mutagen/version"

begin
  require "mutagen/mutagen_ruby"
rescue LoadError
  warn "mutagen native extension not loaded — run `bundle exec rake compile`"
end

module Mutagen
end
