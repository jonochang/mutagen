require "json"

module Mutagen
  class Coverage
    # Read SimpleCov's .resultset.json and convert to a coverage map.
    # Returns Hash<String, Array<Integer>> — file path → covered line numbers
    def self.load(path = nil)
      path ||= default_path
      return {} unless File.exist?(path)

      data = JSON.parse(File.read(path))
      coverage_map = {}

      data.each_value do |result_set|
        coverage = result_set["coverage"] || next

        coverage.each do |file, lines|
          lines = lines["lines"] if lines.is_a?(Hash)
          next unless lines.is_a?(Array)

          covered_lines = []
          lines.each_with_index do |count, index|
            covered_lines << (index + 1) if count && count > 0
          end

          coverage_map[file] ||= []
          coverage_map[file] |= covered_lines
        end
      end

      coverage_map
    end

    def self.default_path
      File.join(Dir.pwd, "coverage", ".resultset.json")
    end
  end
end
