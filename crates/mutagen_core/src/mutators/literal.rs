use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct LiteralMutator;

impl Mutator for LiteralMutator {
    fn category(&self) -> &str {
        "literal"
    }

    fn name(&self) -> &str {
        "literal"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| match node {
                Node::Int(n) => {
                    let begin = n.expression_l.begin;
                    let end = n.expression_l.end;
                    let original =
                        String::from_utf8_lossy(&src[begin..end]).to_string();
                    let replacement = if n.value == "0" {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    };
                    mutations.push(Mutation {
                        id: format!("literal/int@{}:{}", path.display(), begin),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: format!(
                            "literal/int_{}_to_{}",
                            original, replacement
                        ),
                        original,
                        replacement,
                        byte_range: begin..end,
                    });
                }
                Node::Str(n) => {
                    let begin = n.expression_l.begin;
                    let end = n.expression_l.end;
                    let original =
                        String::from_utf8_lossy(&src[begin..end]).to_string();
                    let is_empty = original == "''" || original == "\"\"";
                    let replacement = if is_empty {
                        "'mutagen'".to_string()
                    } else {
                        "''".to_string()
                    };
                    mutations.push(Mutation {
                        id: format!("literal/str@{}:{}", path.display(), begin),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "literal/string".to_string(),
                        original,
                        replacement,
                        byte_range: begin..end,
                    });
                }
                Node::Array(n) => {
                    if n.elements.is_empty() {
                        let begin = n.expression_l.begin;
                        let end = n.expression_l.end;
                        let original =
                            String::from_utf8_lossy(&src[begin..end]).to_string();
                        mutations.push(Mutation {
                            id: format!(
                                "literal/empty_array@{}:{}",
                                path.display(),
                                begin
                            ),
                            file: path.clone(),
                            line: 0,
                            col: 0,
                            operator: "literal/empty_array".to_string(),
                            original,
                            replacement: "[nil]".to_string(),
                            byte_range: begin..end,
                        });
                    }
                }
                _ => {}
            });
        }
        mutations
    }
}
