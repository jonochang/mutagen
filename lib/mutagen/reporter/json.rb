require "json"

module Mutagen
  module Reporter
    class Json
      def report(results, output_path: nil)
        data = {
          "schemaVersion" => "1",
          "thresholds" => { "high" => 80, "low" => 60 },
          "files" => build_files(results)
        }

        json = JSON.pretty_generate(data)

        if output_path
          File.write(output_path, json)
        else
          puts json
        end

        data
      end

      private

      def build_files(results)
        by_file = results.group_by { |r| r.mutation[:file] }
        files = {}

        by_file.each do |file, file_results|
          mutants = file_results.map do |r|
            {
              "id" => r.mutation[:id],
              "mutatorName" => r.mutation[:operator],
              "replacement" => r.mutation[:replacement],
              "location" => {
                "start" => { "line" => r.mutation[:line], "column" => r.mutation[:col] }
              },
              "status" => r.status.capitalize
            }
          end

          files[file] = { "mutants" => mutants }
        end

        files
      end
    end
  end
end
