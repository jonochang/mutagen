require "spec_helper"

RSpec.describe Mutagen::Native do
  describe ".generate_mutations" do
    it "generates arithmetic mutations for binary operators" do
      mutations = Mutagen::Native.generate_mutations("x = 1 + 2", "test.rb")

      arithmetic = mutations.select { |m| m[:operator].start_with?("arithmetic") }
      expect(arithmetic.length).to eq(1)
      expect(arithmetic.first[:original]).to eq("+")
      expect(arithmetic.first[:replacement]).to eq("-")
    end

    it "generates literal mutations for integers" do
      mutations = Mutagen::Native.generate_mutations("x = 42", "test.rb")

      literal = mutations.select { |m| m[:operator].start_with?("literal") }
      expect(literal.length).to eq(1)
      expect(literal.first[:original]).to eq("42")
      expect(literal.first[:replacement]).to eq("0")
    end

    it "generates comparison mutations" do
      mutations = Mutagen::Native.generate_mutations("x > 5", "test.rb")

      comparison = mutations.select { |m| m[:operator].start_with?("comparison") }
      expect(comparison.length).to be >= 1
      operators = comparison.map { |m| m[:replacement] }
      expect(operators).to include(">=")
    end

    it "generates boolean mutations" do
      mutations = Mutagen::Native.generate_mutations("a && b", "test.rb")

      boolean = mutations.select { |m| m[:operator].start_with?("boolean") }
      expect(boolean.length).to eq(1)
      expect(boolean.first[:replacement]).to eq("||")
    end

    it "generates conditional mutations" do
      mutations = Mutagen::Native.generate_mutations("if x > 0\n  y\nend", "test.rb")

      conditional = mutations.select { |m| m[:operator].start_with?("conditional") }
      expect(conditional.length).to be >= 1
    end

    it "returns correct byte ranges for apply_mutation" do
      source = "x = 1 + 2"
      mutations = Mutagen::Native.generate_mutations(source, "test.rb")

      arithmetic = mutations.find { |m| m[:operator].start_with?("arithmetic") }
      result = Mutagen::Native.apply_mutation(
        source, arithmetic[:byte_range_start], arithmetic[:byte_range_end], arithmetic[:replacement]
      )

      expect(result).to eq("x = 1 - 2")
    end

    it "returns an empty array for code with no mutable constructs" do
      mutations = Mutagen::Native.generate_mutations("# just a comment", "test.rb")
      expect(mutations).to eq([])
    end

    it "includes all required keys in each mutation hash" do
      mutations = Mutagen::Native.generate_mutations("x = 1 + 2", "test.rb")
      expect(mutations).not_to be_empty

      required_keys = %i[id file line col operator original replacement byte_range_start byte_range_end]
      mutations.each do |m|
        required_keys.each do |key|
          expect(m).to have_key(key), "Expected mutation to have key :#{key}"
        end
      end
    end
  end

  describe ".apply_mutation" do
    it "replaces the specified byte range with the replacement string" do
      result = Mutagen::Native.apply_mutation("x = 1 + 2", 6, 7, "-")
      expect(result).to eq("x = 1 - 2")
    end

    it "handles string literal mutations" do
      result = Mutagen::Native.apply_mutation('"hello"', 0, 7, '""')
      expect(result).to eq('""')
    end

    it "handles mutations that change code length" do
      result = Mutagen::Native.apply_mutation("x >= y", 2, 4, "<")
      expect(result).to eq("x < y")
    end
  end
end
