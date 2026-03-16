use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct ComparisonMutator;

const REPLACEMENTS: &[(&str, &[&str])] = &[
    (">", &[">=", "<"]),
    ("<", &["<=", ">"]),
    (">=", &[">", "<="]),
    ("<=", &["<", ">="]),
    ("==", &["!="]),
    ("!=", &["=="]),
];

impl Mutator for ComparisonMutator {
    fn category(&self) -> &str {
        "comparison"
    }

    fn name(&self) -> &str {
        "comparison"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| {
                if let Node::Send(send) = node {
                    let method_name = send.method_name.as_str();
                    for &(op, replacements) in REPLACEMENTS {
                        if method_name == op {
                            if let Some(ref loc) = send.selector_l {
                                let begin = loc.begin;
                                let end = loc.end;
                                let original =
                                    String::from_utf8_lossy(&src[begin..end]).to_string();
                                for &replacement in replacements {
                                    mutations.push(Mutation {
                                        id: format!(
                                            "comparison/{}_to_{}@{}:{}",
                                            op,
                                            replacement,
                                            path.display(),
                                            begin
                                        ),
                                        file: path.clone(),
                                        line: 0,
                                        col: 0,
                                        operator: format!(
                                            "comparison/{}_to_{}",
                                            op, replacement
                                        ),
                                        original: original.clone(),
                                        replacement: replacement.to_string(),
                                        byte_range: begin..end,
                                    });
                                }
                            }
                        }
                    }
                }
            });
        }
        mutations
    }
}
