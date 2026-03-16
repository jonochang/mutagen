use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct BooleanMutator;

impl Mutator for BooleanMutator {
    fn category(&self) -> &str {
        "boolean"
    }

    fn name(&self) -> &str {
        "boolean"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| match node {
                Node::And(n) => {
                    let begin = n.operator_l.begin;
                    let end = n.operator_l.end;
                    let original =
                        String::from_utf8_lossy(&src[begin..end]).to_string();
                    mutations.push(Mutation {
                        id: format!("boolean/and_to_or@{}:{}", path.display(), begin),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "boolean/and_to_or".to_string(),
                        original,
                        replacement: "||".to_string(),
                        byte_range: begin..end,
                    });
                }
                Node::Or(n) => {
                    let begin = n.operator_l.begin;
                    let end = n.operator_l.end;
                    let original =
                        String::from_utf8_lossy(&src[begin..end]).to_string();
                    mutations.push(Mutation {
                        id: format!("boolean/or_to_and@{}:{}", path.display(), begin),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "boolean/or_to_and".to_string(),
                        original,
                        replacement: "&&".to_string(),
                        byte_range: begin..end,
                    });
                }
                Node::True(n) => {
                    let begin = n.expression_l.begin;
                    let end = n.expression_l.end;
                    mutations.push(Mutation {
                        id: format!(
                            "boolean/true_to_false@{}:{}",
                            path.display(),
                            begin
                        ),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "boolean/true_to_false".to_string(),
                        original: "true".to_string(),
                        replacement: "false".to_string(),
                        byte_range: begin..end,
                    });
                }
                Node::False(n) => {
                    let begin = n.expression_l.begin;
                    let end = n.expression_l.end;
                    mutations.push(Mutation {
                        id: format!(
                            "boolean/false_to_true@{}:{}",
                            path.display(),
                            begin
                        ),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "boolean/false_to_true".to_string(),
                        original: "false".to_string(),
                        replacement: "true".to_string(),
                        byte_range: begin..end,
                    });
                }
                _ => {}
            });
        }
        mutations
    }
}
