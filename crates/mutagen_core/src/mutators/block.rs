use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct BlockMutator;

const METHOD_SWAPS: &[(&str, &str)] = &[
    ("each", "map"),
    ("map", "each"),
    ("select", "reject"),
    ("reject", "select"),
    ("detect", "select"),
    ("find", "select"),
    ("collect", "each"),
    ("flat_map", "map"),
];

impl Mutator for BlockMutator {
    fn category(&self) -> &str {
        "block"
    }

    fn name(&self) -> &str {
        "block"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| {
                if let Node::Block(n) = node {
                    // Method swap: each ↔ map, select ↔ reject, etc.
                    if let Node::Send(send) = n.call.as_ref() {
                        let method = send.method_name.as_str();
                        for &(from, to) in METHOD_SWAPS {
                            if method == from {
                                if let Some(ref loc) = send.selector_l {
                                    let begin = loc.begin;
                                    let end = loc.end;
                                    let original =
                                        String::from_utf8_lossy(&src[begin..end])
                                            .to_string();
                                    mutations.push(Mutation {
                                        id: format!(
                                            "block/{}_to_{}@{}:{}",
                                            from,
                                            to,
                                            path.display(),
                                            begin
                                        ),
                                        file: path.clone(),
                                        line: 0,
                                        col: 0,
                                        operator: format!(
                                            "block/{}_to_{}",
                                            from, to
                                        ),
                                        original,
                                        replacement: to.to_string(),
                                        byte_range: begin..end,
                                    });
                                }
                                break;
                            }
                        }
                    }

                    // Remove block body
                    if let Some(ref body) = n.body {
                        let (begin, end) = walk::node_expression(body);
                        let original =
                            String::from_utf8_lossy(&src[begin..end]).to_string();
                        mutations.push(Mutation {
                            id: format!(
                                "block/remove_body@{}:{}",
                                path.display(),
                                begin
                            ),
                            file: path.clone(),
                            line: 0,
                            col: 0,
                            operator: "block/remove_body".to_string(),
                            original,
                            replacement: "nil".to_string(),
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
    fn mutates_each_to_map() {
        let source = b"[1,2].each { |x| puts x }".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = BlockMutator.generate(&sf);

        let swap = mutations.iter().find(|m| m.operator == "block/each_to_map");
        assert!(swap.is_some());
        assert_eq!(swap.unwrap().replacement, "map");
    }

    #[test]
    fn mutates_map_to_each() {
        let source = b"[1,2].map { |x| x * 2 }".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = BlockMutator.generate(&sf);

        let swap = mutations.iter().find(|m| m.operator == "block/map_to_each");
        assert!(swap.is_some());
        assert_eq!(swap.unwrap().replacement, "each");
    }

    #[test]
    fn mutates_select_to_reject() {
        let source = b"[1,2].select { |x| x > 1 }".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = BlockMutator.generate(&sf);

        let swap = mutations
            .iter()
            .find(|m| m.operator == "block/select_to_reject");
        assert!(swap.is_some());
        assert_eq!(swap.unwrap().replacement, "reject");
    }

    #[test]
    fn removes_block_body() {
        let source = b"[1,2].each { |x| puts x }".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = BlockMutator.generate(&sf);

        let remove = mutations
            .iter()
            .find(|m| m.operator == "block/remove_body");
        assert!(remove.is_some());
        assert_eq!(remove.unwrap().replacement, "nil");
    }

    #[test]
    fn no_swap_for_unknown_method() {
        let source = b"[1,2].foo { |x| x }".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = BlockMutator.generate(&sf);

        // Should still have remove_body but no method swap
        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].operator, "block/remove_body");
    }

    #[test]
    fn no_body_removal_for_empty_block() {
        let source = b"[1,2].each { }".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = BlockMutator.generate(&sf);

        // Only the method swap, no body removal
        let remove = mutations
            .iter()
            .find(|m| m.operator == "block/remove_body");
        assert!(remove.is_none());
    }
}
