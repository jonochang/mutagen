module Mutagen
  module TestRunner
    class Base
      attr_reader :baseline_duration

      def run_tests(test_files, env: {})
        raise NotImplementedError
      end

      def baseline_run
        raise NotImplementedError
      end
    end
  end
end
