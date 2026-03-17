require "spec_helper"
require "mutagen/worker_pool"
require "mutagen/test_runner/base"
require "tempfile"

RSpec.describe Mutagen::WorkerPool do
  let(:test_runner) do
    runner = Mutagen::TestRunner::Base.new
    allow(runner).to receive(:baseline_duration).and_return(10.0)
    runner
  end

  def make_mutation(file:, mutated_source:, id: "test_mutation")
    {
      id: id,
      file: file,
      line: 1,
      col: 0,
      operator: "test",
      original: "x",
      replacement: "y",
      mutated_source: mutated_source,
      test_files: nil
    }
  end

  describe "file backup and restore" do
    it "restores the original file after mutation" do
      tmpfile = Tempfile.new(["test", ".rb"])
      original_content = "# original content"
      tmpfile.write(original_content)
      tmpfile.close

      mutation = make_mutation(file: tmpfile.path, mutated_source: "# MUTATED content")
      pool = Mutagen::WorkerPool.new(workers: 1, test_runner: test_runner)

      allow(Process).to receive(:fork) do |&block|
        pid = Kernel.fork { exit!(0) }
        pid
      end

      pool.run([mutation])

      expect(File.read(tmpfile.path)).to eq(original_content)
    ensure
      tmpfile&.unlink
      backup = "#{tmpfile&.path}.mutagen_backup"
      File.delete(backup) if backup && File.exist?(backup)
    end

    it "restores the original file even when the test process errors" do
      tmpfile = Tempfile.new(["test", ".rb"])
      original_content = "# should be restored"
      tmpfile.write(original_content)
      tmpfile.close

      mutation = make_mutation(file: tmpfile.path, mutated_source: "# broken")
      pool = Mutagen::WorkerPool.new(workers: 1, test_runner: test_runner)

      # Simulate fork raising an error
      allow(Process).to receive(:fork).and_raise(RuntimeError, "fork failed")

      pool.run([mutation])

      expect(File.read(tmpfile.path)).to eq(original_content)
    ensure
      tmpfile&.unlink
      backup = "#{tmpfile&.path}.mutagen_backup"
      File.delete(backup) if backup && File.exist?(backup)
    end
  end

  describe "integration: real fork execution" do
    it "kills a mutation when the forked command fails" do
      tmpfile = Tempfile.new(["test", ".rb"])
      tmpfile.write("# original")
      tmpfile.close

      mutation = make_mutation(file: tmpfile.path, mutated_source: "# mutated")
      pool = Mutagen::WorkerPool.new(workers: 1, test_runner: test_runner)

      # Override exec in the forked child to exit with failure
      allow(Process).to receive(:fork) do |&block|
        pid = Kernel.fork do
          # Exit with code 1 (test failure) instead of running rspec
          exit!(1)
        end
        pid
      end

      results = pool.run([mutation])
      expect(results.first.status).to eq("killed")
      expect(File.read(tmpfile.path)).to eq("# original")
    ensure
      tmpfile&.unlink
      backup = "#{tmpfile&.path}.mutagen_backup"
      File.delete(backup) if backup && File.exist?(backup)
    end

    it "marks survived when the forked command succeeds" do
      tmpfile = Tempfile.new(["test", ".rb"])
      tmpfile.write("# original")
      tmpfile.close

      mutation = make_mutation(file: tmpfile.path, mutated_source: "# mutated")
      pool = Mutagen::WorkerPool.new(workers: 1, test_runner: test_runner)

      allow(Process).to receive(:fork) do |&block|
        pid = Kernel.fork do
          exit!(0)
        end
        pid
      end

      results = pool.run([mutation])
      expect(results.first.status).to eq("survived")
      expect(File.read(tmpfile.path)).to eq("# original")
    ensure
      tmpfile&.unlink
      backup = "#{tmpfile&.path}.mutagen_backup"
      File.delete(backup) if backup && File.exist?(backup)
    end

    it "marks error when command is not found (exit 127)" do
      tmpfile = Tempfile.new(["test", ".rb"])
      tmpfile.write("# original")
      tmpfile.close

      mutation = make_mutation(file: tmpfile.path, mutated_source: "# mutated")
      pool = Mutagen::WorkerPool.new(workers: 1, test_runner: test_runner)

      allow(Process).to receive(:fork) do |&block|
        pid = Kernel.fork do
          exit!(127)
        end
        pid
      end

      results = pool.run([mutation])
      expect(results.first.status).to eq("error")
    ensure
      tmpfile&.unlink
      backup = "#{tmpfile&.path}.mutagen_backup"
      File.delete(backup) if backup && File.exist?(backup)
    end

    it "marks error when command is not executable (exit 126)" do
      tmpfile = Tempfile.new(["test", ".rb"])
      tmpfile.write("# original")
      tmpfile.close

      mutation = make_mutation(file: tmpfile.path, mutated_source: "# mutated")
      pool = Mutagen::WorkerPool.new(workers: 1, test_runner: test_runner)

      allow(Process).to receive(:fork) do |&block|
        pid = Kernel.fork do
          exit!(126)
        end
        pid
      end

      results = pool.run([mutation])
      expect(results.first.status).to eq("error")
    ensure
      tmpfile&.unlink
      backup = "#{tmpfile&.path}.mutagen_backup"
      File.delete(backup) if backup && File.exist?(backup)
    end
  end

  describe "parallel execution" do
    it "processes multiple mutations across workers" do
      tmpfile = Tempfile.new(["test", ".rb"])
      tmpfile.write("# original")
      tmpfile.close

      mutations = 3.times.map do |i|
        make_mutation(file: tmpfile.path, mutated_source: "# mutated #{i}", id: "mutation_#{i}")
      end

      pool = Mutagen::WorkerPool.new(workers: 2, test_runner: test_runner)

      allow(Process).to receive(:fork) do |&block|
        pid = Kernel.fork { exit!(1) }
        pid
      end

      results = pool.run(mutations)
      expect(results.length).to eq(3)
      expect(results.map(&:status)).to all(eq("killed"))
      # File should be restored
      expect(File.read(tmpfile.path)).to eq("# original")
    ensure
      tmpfile&.unlink
      backup = "#{tmpfile&.path}.mutagen_backup"
      File.delete(backup) if backup && File.exist?(backup)
    end
  end
end
