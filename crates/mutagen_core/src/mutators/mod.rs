pub mod arithmetic;
pub mod assignment;
pub mod block;
pub mod boolean;
pub mod comparison;
pub mod conditional;
pub mod literal;
pub mod regex;
pub mod return_val;
pub mod statement;
mod walk;

pub use walk::walk_children;

use crate::parser::SourceFile;
use std::ops::Range;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Mutation {
    pub id: String,
    pub file: PathBuf,
    pub line: u32,
    pub col: u32,
    pub operator: String,
    pub original: String,
    pub replacement: String,
    pub byte_range: Range<usize>,
}

pub trait Mutator: Send + Sync {
    fn category(&self) -> &str;
    fn name(&self) -> &str;
    fn generate(&self, source: &SourceFile) -> Vec<Mutation>;
}

pub struct MutatorRegistry {
    mutators: Vec<Box<dyn Mutator>>,
}

impl MutatorRegistry {
    pub fn new() -> Self {
        Self {
            mutators: Vec::new(),
        }
    }

    pub fn register(&mut self, mutator: Box<dyn Mutator>) {
        self.mutators.push(mutator);
    }

    pub fn generate_all(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations: Vec<Mutation> = self.mutators
            .iter()
            .flat_map(|m| m.generate(source))
            .collect();

        // Fill in line/col from byte offsets
        for m in &mut mutations {
            let (line, col) = source.byte_offset_to_line_col(m.byte_range.start);
            m.line = line;
            m.col = col;
        }

        mutations
    }

    pub fn default_registry() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(arithmetic::ArithmeticMutator));
        registry.register(Box::new(comparison::ComparisonMutator));
        registry.register(Box::new(boolean::BooleanMutator));
        registry.register(Box::new(conditional::ConditionalMutator));
        registry.register(Box::new(literal::LiteralMutator));
        registry.register(Box::new(assignment::AssignmentMutator));
        registry.register(Box::new(return_val::ReturnMutator));
        registry.register(Box::new(statement::StatementMutator));
        registry.register(Box::new(block::BlockMutator));
        registry.register(Box::new(regex::RegexMutator));
        registry
    }
}

impl Default for MutatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
