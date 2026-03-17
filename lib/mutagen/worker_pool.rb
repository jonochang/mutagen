module Mutagen
  class WorkerPool
    Result = Struct.new(:mutation, :status, :killing_test, :duration_ms, keyword_init: true)

    def initialize(workers:, test_runner:, timeout_multiplier: 3.0)
      @workers = workers
      @test_runner = test_runner
      @timeout_multiplier = timeout_multiplier
      @baseline_duration = test_runner.baseline_duration || 10.0
    end

    def run(mutations)
      results = []
      queue = mutations.dup

      if @workers <= 1
        queue.each_with_index do |mutation, idx|
          results << execute_mutation(mutation, worker_id: 0)
        end
      else
        run_parallel(queue, results)
      end

      results
    end

    private

    def run_parallel(queue, results)
      mutex = Mutex.new
      # Per-file locks to prevent concurrent mutation of the same file
      @file_locks = Hash.new { |h, k| h[k] = Mutex.new }
      @file_locks_mutex = Mutex.new
      mutation_queue = Queue.new
      queue.each { |m| mutation_queue << m }

      threads = @workers.times.map do |worker_id|
        Thread.new do
          loop do
            mutation = begin
              mutation_queue.pop(true)
            rescue ThreadError
              break
            end

            file_lock = @file_locks_mutex.synchronize { @file_locks[mutation[:file]] }
            file_lock.synchronize do
              result = execute_mutation(mutation, worker_id: worker_id)
              mutex.synchronize { results << result }
            end
          end
        end
      end

      threads.each(&:join)
    end

    def execute_mutation(mutation, worker_id:)
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      timeout = @baseline_duration * @timeout_multiplier

      env = {
        "MUTAGEN_WORKER" => worker_id.to_s,
        "MUTAGEN_ID" => mutation[:id]
      }

      original_file = mutation[:file]
      original_source = File.read(original_file)

      begin
        # Replace original file with mutated source
        File.write(original_file, mutation[:mutated_source])

        pid = Process.fork do
          ENV.update(env)
          exec("bundle", "exec", "rspec", "--fail-fast", "--format", "progress",
               *Array(mutation[:test_files]))
        end

        if pid
          thread = Thread.new { Process.wait2(pid) }
          unless thread.join(timeout)
            Process.kill("TERM", pid)
            thread.join(5) || Process.kill("KILL", pid)
            duration = ((Process.clock_gettime(Process::CLOCK_MONOTONIC) - start) * 1000).round
            return Result.new(
              mutation: mutation,
              status: "timeout",
              killing_test: nil,
              duration_ms: duration
            )
          end

          _, status = thread.value
          duration = ((Process.clock_gettime(Process::CLOCK_MONOTONIC) - start) * 1000).round

          mutation_status = if status.success?
            "survived"
          elsif status.exitstatus == 127 || status.exitstatus == 126
            # 127 = command not found, 126 = permission denied / not executable
            "error"
          elsif status.termsig
            "error"
          else
            "killed"
          end

          Result.new(
            mutation: mutation,
            status: mutation_status,
            killing_test: nil,
            duration_ms: duration
          )
        end
      rescue => e
        duration = ((Process.clock_gettime(Process::CLOCK_MONOTONIC) - start) * 1000).round
        Result.new(
          mutation: mutation,
          status: "error",
          killing_test: nil,
          duration_ms: duration
        )
      ensure
        # Always restore the original file
        File.write(original_file, original_source)
      end
    end
  end
end
