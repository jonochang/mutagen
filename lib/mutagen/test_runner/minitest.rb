require_relative "base"

module Mutagen
  module TestRunner
    class Minitest < Base
      def initialize
        @baseline_duration = nil
      end

      def baseline_run
        start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
        result = run_command("bundle", "exec", "ruby", "-e", "ARGV.each { |f| require f }")
        @baseline_duration = Process.clock_gettime(Process::CLOCK_MONOTONIC) - start
        result
      end

      def run_tests(test_files, env: {})
        args = ["bundle", "exec", "ruby", "-e", "ARGV.each { |f| require f }"]
        args.concat(test_files)
        run_command(*args, env: env)
      end

      private

      def run_command(*args, env: {})
        out_r, out_w = IO.pipe
        err_r, err_w = IO.pipe

        pid = Process.spawn(env, *args, out: out_w, err: err_w)
        out_w.close
        err_w.close

        stdout = out_r.read
        stderr = err_r.read
        out_r.close
        err_r.close

        _, status = Process.wait2(pid)

        {
          success: status.success?,
          exit_code: status.exitstatus,
          stdout: stdout,
          stderr: stderr
        }
      end
    end
  end
end
