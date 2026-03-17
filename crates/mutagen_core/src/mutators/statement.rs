use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct StatementMutator;

impl Mutator for StatementMutator {
    fn category(&self) -> &str {
        "statement"
    }

    fn name(&self) -> &str {
        "statement"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| {
                if let Node::Begin(n) = node {
                    // Only remove statements when there are at least 2
                    if n.statements.len() >= 2 {
                        for (i, stmt) in n.statements.iter().enumerate() {
                            let (begin, end) = walk::node_expression(stmt);
                            let original =
                                String::from_utf8_lossy(&src[begin..end]).to_string();
                            mutations.push(Mutation {
                                id: format!(
                                    "statement/remove_{}@{}:{}",
                                    i,
                                    path.display(),
                                    begin
                                ),
                                file: path.clone(),
                                line: 0,
                                col: 0,
                                operator: "statement/remove".to_string(),
                                original,
                                replacement: "nil".to_string(),
                                byte_range: begin..end,
                            });
                        }
                    }
                }
            });
        }
        mutations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SourceFile;
    use std::path::PathBuf;

    #[test]
    fn removes_statements_in_method_body() {
        let source = b"def foo\n  a = 1\n  b = 2\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = StatementMutator.generate(&sf);

        assert_eq!(mutations.len(), 2);
        assert_eq!(mutations[0].operator, "statement/remove");
        assert_eq!(mutations[0].original, "a = 1");
        assert_eq!(mutations[0].replacement, "nil");
        assert_eq!(mutations[1].original, "b = 2");
    }

    #[test]
    fn skips_single_statement_body() {
        let source = b"def foo\n  a = 1\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = StatementMutator.generate(&sf);

        assert!(mutations.is_empty());
    }

    #[test]
    fn handles_three_statements() {
        let source = b"def foo\n  a = 1\n  b = 2\n  c = 3\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = StatementMutator.generate(&sf);

        assert_eq!(mutations.len(), 3);
    }

    #[test]
    fn no_mutations_for_empty_method() {
        let source = b"def foo\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = StatementMutator.generate(&sf);

        assert!(mutations.is_empty());
    }
}
