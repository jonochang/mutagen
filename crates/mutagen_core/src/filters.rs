use crate::mutators::Mutation;
use crate::parser::SourceFile;
use lib_ruby_parser::Node;
use std::collections::HashSet;

/// Remove mutations with identical file + byte_range + replacement.
pub fn deduplicate(mutations: Vec<Mutation>) -> Vec<Mutation> {
    let mut seen = HashSet::new();
    mutations
        .into_iter()
        .filter(|m| {
            let key = format!(
                "{}:{}..{}:{}",
                m.file.display(),
                m.byte_range.start,
                m.byte_range.end,
                m.replacement
            );
            seen.insert(key)
        })
        .collect()
}

/// Remove mutations that fall within unreachable code (after return/raise/exit/break/next).
pub fn filter_dead_code(mutations: Vec<Mutation>, source: &SourceFile) -> Vec<Mutation> {
    let unreachable = find_unreachable_ranges(source);
    if unreachable.is_empty() {
        return mutations;
    }

    mutations
        .into_iter()
        .filter(|m| {
            !unreachable.iter().any(|r| {
                m.byte_range.start >= r.start && m.byte_range.end <= r.end
            })
        })
        .collect()
}

/// Remove mutations on `require` / `require_relative` calls.
pub fn filter_require_statements(mutations: Vec<Mutation>, source: &SourceFile) -> Vec<Mutation> {
    let require_ranges = find_require_ranges(source);
    if require_ranges.is_empty() {
        return mutations;
    }

    mutations
        .into_iter()
        .filter(|m| {
            !require_ranges.iter().any(|r| {
                m.byte_range.start >= r.start && m.byte_range.end <= r.end
            })
        })
        .collect()
}

fn find_unreachable_ranges(source: &SourceFile) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    if let Some(ref ast) = source.result.ast {
        // walk_all visits every node in the tree
        crate::mutators::walk::walk_all(ast, &mut |node| {
            if let Node::Begin(n) = node {
                let mut found_terminator = false;
                for stmt in &n.statements {
                    if found_terminator {
                        let (begin, end) = crate::mutators::walk::node_expression(stmt);
                        ranges.push(begin..end);
                    } else if is_terminator(stmt) {
                        found_terminator = true;
                    }
                }
            }
        });
    }
    ranges
}

fn is_terminator(node: &Node) -> bool {
    match node {
        Node::Return(_) | Node::Break(_) | Node::Next(_) => true,
        Node::Send(s) => matches!(
            s.method_name.as_str(),
            "raise" | "fail" | "exit" | "abort" | "exit!"
        ),
        _ => false,
    }
}

fn find_require_ranges(source: &SourceFile) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    if let Some(ref ast) = source.result.ast {
        crate::mutators::walk::walk_all(ast, &mut |node| {
            if let Node::Send(send) = node {
                let method = send.method_name.as_str();
                if method == "require" || method == "require_relative" {
                    let begin = send.expression_l.begin;
                    let end = send.expression_l.end;
                    ranges.push(begin..end);
                }
            }
        });
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutators::{Mutation, MutatorRegistry};
    use std::path::PathBuf;

    fn parse_and_generate(code: &str) -> (SourceFile, Vec<Mutation>) {
        let sf = SourceFile::parse(PathBuf::from("test.rb"), code.as_bytes().to_vec());
        let registry = MutatorRegistry::default_registry();
        let mutations = registry.generate_all(&sf);
        (sf, mutations)
    }

    #[test]
    fn deduplicates_identical_mutations() {
        let m1 = Mutation {
            id: "a".to_string(),
            file: PathBuf::from("test.rb"),
            line: 1,
            col: 1,
            operator: "test".to_string(),
            original: "x".to_string(),
            replacement: "y".to_string(),
            byte_range: 0..1,
        };
        let m2 = m1.clone();
        let m3 = Mutation {
            id: "b".to_string(),
            replacement: "z".to_string(),
            ..m1.clone()
        };

        let result = deduplicate(vec![m1, m2, m3]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filters_dead_code_after_return() {
        let code = "def foo\n  return 1\n  x + y\nend";
        let (sf, mutations) = parse_and_generate(code);

        let before = mutations.len();
        let after = filter_dead_code(mutations, &sf);
        // x + y is after return, so arithmetic mutation on it should be removed
        assert!(
            after.len() < before,
            "expected dead code filtering to remove mutations, before={} after={}",
            before,
            after.len()
        );
    }

    #[test]
    fn keeps_mutations_before_return() {
        let code = "def foo\n  x + y\n  return 1\nend";
        let (sf, mutations) = parse_and_generate(code);

        let before = mutations.len();
        let after = filter_dead_code(mutations, &sf);
        let arithmetic_count = after
            .iter()
            .filter(|m| m.operator.starts_with("arithmetic"))
            .count();
        assert!(arithmetic_count > 0);
        assert_eq!(after.len(), before);
    }

    #[test]
    fn filters_dead_code_after_raise() {
        let code = "def foo\n  raise 'error'\n  x + y\nend";
        let (sf, mutations) = parse_and_generate(code);

        let before = mutations.len();
        let after = filter_dead_code(mutations, &sf);
        assert!(after.len() < before);
    }

    #[test]
    fn filters_require_mutations() {
        let code = "require 'json'\nx + y";
        let (sf, mutations) = parse_and_generate(code);

        let before = mutations.len();
        let after = filter_require_statements(mutations, &sf);
        let require_mutations = before - after.len();
        assert!(
            require_mutations > 0,
            "expected require mutations to be filtered"
        );
    }

    #[test]
    fn keeps_non_require_mutations() {
        let code = "x + y";
        let (sf, mutations) = parse_and_generate(code);

        let before = mutations.len();
        let after = filter_require_statements(mutations, &sf);
        assert_eq!(after.len(), before);
    }
}
