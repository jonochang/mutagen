use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct ConditionalMutator;

impl Mutator for ConditionalMutator {
    fn category(&self) -> &str {
        "conditional"
    }

    fn name(&self) -> &str {
        "conditional"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| {
                if let Node::If(n) = node {
                    let (cond_begin, cond_end) = walk::node_expression(&n.cond);
                    let original =
                        String::from_utf8_lossy(&src[cond_begin..cond_end]).to_string();

                    // condition → true
                    mutations.push(Mutation {
                        id: format!(
                            "conditional/condition_to_true@{}:{}",
                            path.display(),
                            cond_begin
                        ),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "conditional/condition_to_true".to_string(),
                        original: original.clone(),
                        replacement: "true".to_string(),
                        byte_range: cond_begin..cond_end,
                    });

                    // condition → false
                    mutations.push(Mutation {
                        id: format!(
                            "conditional/condition_to_false@{}:{}",
                            path.display(),
                            cond_begin
                        ),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "conditional/condition_to_false".to_string(),
                        original,
                        replacement: "false".to_string(),
                        byte_range: cond_begin..cond_end,
                    });

                    // Remove else branch if present
                    if n.if_false.is_some() {
                        if let Some(ref else_l) = n.else_l {
                            if let Some(ref end_l) = n.end_l {
                                let else_begin = else_l.begin;
                                let else_end = end_l.begin;
                                mutations.push(Mutation {
                                    id: format!(
                                        "conditional/remove_else@{}:{}",
                                        path.display(),
                                        else_begin
                                    ),
                                    file: path.clone(),
                                    line: 0,
                                    col: 0,
                                    operator: "conditional/remove_else".to_string(),
                                    original: String::from_utf8_lossy(
                                        &src[else_begin..else_end],
                                    )
                                    .to_string(),
                                    replacement: "\n".to_string(),
                                    byte_range: else_begin..else_end,
                                });
                            }
                        }
                    }
                }
            });
        }
        mutations
    }
}
