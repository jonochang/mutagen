use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct ArithmeticMutator;

const REPLACEMENTS: &[(&str, &[&str])] = &[
    ("+", &["-"]),
    ("-", &["+"]),
    ("*", &["/"]),
    ("/", &["*"]),
    ("%", &["*"]),
];

impl Mutator for ArithmeticMutator {
    fn category(&self) -> &str {
        "arithmetic"
    }

    fn name(&self) -> &str {
        "arithmetic"
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
                                            "arithmetic/{}_to_{}@{}:{}",
                                            op,
                                            replacement,
                                            path.display(),
                                            begin
                                        ),
                                        file: path.clone(),
                                        line: 0,
                                        col: 0,
                                        operator: format!(
                                            "arithmetic/{}_to_{}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SourceFile;
    use std::path::PathBuf;

    #[test]
    fn mutates_plus_to_minus() {
        let source = b"a + b".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutator = ArithmeticMutator;
        let mutations = mutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].original, "+");
        assert_eq!(mutations[0].replacement, "-");
        assert_eq!(mutations[0].operator, "arithmetic/+_to_-");
    }

    #[test]
    fn mutates_minus_to_plus() {
        let source = b"a - b".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutator = ArithmeticMutator;
        let mutations = mutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].replacement, "+");
    }

    #[test]
    fn mutates_multiply_to_divide() {
        let source = b"a * b".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutator = ArithmeticMutator;
        let mutations = mutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].replacement, "/");
    }

    #[test]
    fn no_mutations_for_non_arithmetic() {
        let source = b"a.foo(b)".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutator = ArithmeticMutator;
        let mutations = mutator.generate(&sf);

        assert!(mutations.is_empty());
    }

    #[test]
    fn finds_nested_arithmetic() {
        let source = b"def calc\n  a + b\nend".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutator = ArithmeticMutator;
        let mutations = mutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].original, "+");
        assert_eq!(mutations[0].replacement, "-");
    }
}
