use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct ReturnMutator;

impl Mutator for ReturnMutator {
    fn category(&self) -> &str {
        "return"
    }

    fn name(&self) -> &str {
        "return"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| {
                if let Node::Return(n) = node {
                    if !n.args.is_empty() {
                        // return expr → return nil
                        let value_begin = n.keyword_l.end;
                        let value_end = n.expression_l.end;
                        let original =
                            String::from_utf8_lossy(&src[value_begin..value_end]).to_string();
                        mutations.push(Mutation {
                            id: format!(
                                "return/value_to_nil@{}:{}",
                                path.display(),
                                n.keyword_l.begin
                            ),
                            file: path.clone(),
                            line: 0,
                            col: 0,
                            operator: "return/value_to_nil".to_string(),
                            original,
                            replacement: " nil".to_string(),
                            byte_range: value_begin..value_end,
                        });

                        // return expr → expr (remove return keyword)
                        let begin = n.expression_l.begin;
                        let end = n.expression_l.end;
                        let original =
                            String::from_utf8_lossy(&src[begin..end]).to_string();
                        let (val_begin, val_end) = walk::node_expression(&n.args[0]);
                        let value =
                            String::from_utf8_lossy(&src[val_begin..val_end]).to_string();
                        mutations.push(Mutation {
                            id: format!(
                                "return/remove_return@{}:{}",
                                path.display(),
                                begin
                            ),
                            file: path.clone(),
                            line: 0,
                            col: 0,
                            operator: "return/remove_return".to_string(),
                            original,
                            replacement: value,
                            byte_range: begin..end,
                        });
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
    fn mutates_return_value_to_nil() {
        let source = b"def foo\n  return 42\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = ReturnMutator.generate(&sf);

        assert_eq!(mutations.len(), 2);
        assert_eq!(mutations[0].operator, "return/value_to_nil");
        assert_eq!(mutations[0].replacement, " nil");
    }

    #[test]
    fn mutates_remove_return_keyword() {
        let source = b"def foo\n  return 42\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = ReturnMutator.generate(&sf);

        assert_eq!(mutations[1].operator, "return/remove_return");
        assert_eq!(mutations[1].replacement, "42");
    }

    #[test]
    fn no_mutation_for_bare_return() {
        let source = b"def foo\n  return\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = ReturnMutator.generate(&sf);

        assert!(mutations.is_empty());
    }

    #[test]
    fn mutates_return_with_variable() {
        let source = b"def foo\n  return bar\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = ReturnMutator.generate(&sf);

        assert_eq!(mutations.len(), 2);
        assert_eq!(mutations[1].replacement, "bar");
    }
}
