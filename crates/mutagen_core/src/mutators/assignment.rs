use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct AssignmentMutator;

const OP_REPLACEMENTS: &[(&str, &str)] = &[
    ("+", "-"),
    ("-", "+"),
    ("*", "/"),
    ("/", "*"),
    ("%", "*"),
    ("**", "*"),
    ("<<", ">>"),
    (">>", "<<"),
    ("&", "|"),
    ("|", "&"),
    ("^", "&"),
];

impl Mutator for AssignmentMutator {
    fn category(&self) -> &str {
        "assignment"
    }

    fn name(&self) -> &str {
        "assignment"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| match node {
                Node::OpAsgn(n) => {
                    let op = n.operator.as_str();
                    for &(from, to) in OP_REPLACEMENTS {
                        if op == from {
                            let begin = n.operator_l.begin;
                            let end = n.operator_l.end;
                            let original =
                                String::from_utf8_lossy(&src[begin..end]).to_string();
                            let replacement = format!("{}=", to);
                            mutations.push(Mutation {
                                id: format!(
                                    "assignment/{}=_to_{}=@{}:{}",
                                    from,
                                    to,
                                    path.display(),
                                    begin
                                ),
                                file: path.clone(),
                                line: 0,
                                col: 0,
                                operator: format!("assignment/{}=_to_{}=", from, to),
                                original,
                                replacement,
                                byte_range: begin..end,
                            });
                            break;
                        }
                    }
                }
                Node::AndAsgn(n) => {
                    let begin = n.operator_l.begin;
                    let end = n.operator_l.end;
                    let original =
                        String::from_utf8_lossy(&src[begin..end]).to_string();
                    mutations.push(Mutation {
                        id: format!(
                            "assignment/&&=_to_||=@{}:{}",
                            path.display(),
                            begin
                        ),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "assignment/&&=_to_||=".to_string(),
                        original,
                        replacement: "||=".to_string(),
                        byte_range: begin..end,
                    });
                }
                Node::OrAsgn(n) => {
                    let begin = n.operator_l.begin;
                    let end = n.operator_l.end;
                    let original =
                        String::from_utf8_lossy(&src[begin..end]).to_string();
                    mutations.push(Mutation {
                        id: format!(
                            "assignment/||=_to_&&=@{}:{}",
                            path.display(),
                            begin
                        ),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "assignment/||=_to_&&=".to_string(),
                        original,
                        replacement: "&&=".to_string(),
                        byte_range: begin..end,
                    });
                }
                _ => {}
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
    fn mutates_plus_equals_to_minus_equals() {
        let source = b"a += 1".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = AssignmentMutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].original, "+=");
        assert_eq!(mutations[0].replacement, "-=");
        assert_eq!(mutations[0].operator, "assignment/+=_to_-=");
    }

    #[test]
    fn mutates_minus_equals_to_plus_equals() {
        let source = b"a -= 1".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = AssignmentMutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].replacement, "+=");
    }

    #[test]
    fn mutates_and_asgn_to_or_asgn() {
        let source = b"a &&= true".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = AssignmentMutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].original, "&&=");
        assert_eq!(mutations[0].replacement, "||=");
    }

    #[test]
    fn mutates_or_asgn_to_and_asgn() {
        let source = b"a ||= 'default'".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = AssignmentMutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].original, "||=");
        assert_eq!(mutations[0].replacement, "&&=");
    }

    #[test]
    fn mutates_shift_left_equals() {
        let source = b"a <<= 2".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = AssignmentMutator.generate(&sf);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].replacement, ">>=");
    }

    #[test]
    fn no_mutations_for_plain_assignment() {
        let source = b"a = 1".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = AssignmentMutator.generate(&sf);

        assert!(mutations.is_empty());
    }
}
