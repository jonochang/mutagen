module Mutagen
  module Reporter
    class Console
      def report(results, total_duration: nil)
        killed = results.count { |r| r.status == "killed" }
        survived = results.count { |r| r.status == "survived" }
        timeout = results.count { |r| r.status == "timeout" }
        errored = results.count { |r| r.status == "error" }
        scoreable = killed + survived
        score = scoreable > 0 ? (killed.to_f / scoreable * 100).round(1) : 0.0

        puts ""
        puts "Mutation Score: #{score}% (#{killed}/#{scoreable} killed)"
        puts ""
        puts format("%-40s %10s %8s %10s %7s", "File", "Mutations", "Killed", "Survived", "Score")
        puts "-" * 77

        by_file = results.group_by { |r| r.mutation[:file] }
        by_file.sort_by { |f, _| f }.each do |file, file_results|
          fk = file_results.count { |r| r.status == "killed" }
          fs = file_results.count { |r| r.status == "survived" }
          ft = fk + fs
          fscore = ft > 0 ? (fk.to_f / ft * 100).round(1) : 0.0
          puts format("%-40s %10d %8d %10d %6.1f%%", file, ft, fk, fs, fscore)
        end

        puts ""
        extras = []
        extras << "#{timeout} timed out" if timeout > 0
        extras << "#{errored} errored" if errored > 0
        puts extras.join(", ") + " (excluded from score)" unless extras.empty?

        if total_duration
          puts ""
          puts "Completed in #{total_duration.round(1)}s"
        end

        score
      end
    end
  end
end
