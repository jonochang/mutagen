use crate::mutators::{Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::nodes::Send;
use lib_ruby_parser::Node;
use std::path::PathBuf;

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
            visit_node(ast, &source.path, &source.source, &mut mutations);
        }
        mutations
    }
}

fn visit_send(send: &Send, path: &PathBuf, source: &[u8], mutations: &mut Vec<Mutation>) {
    let method_name = send.method_name.as_str();

    for &(op, replacements) in REPLACEMENTS {
        if method_name == op {
            // Binary operators like +, -, *, / use selector_l for the operator location
            if let Some(ref loc) = send.selector_l {
                let begin = loc.begin;
                let end = loc.end;
                let original = String::from_utf8_lossy(&source[begin..end]).to_string();

                for &replacement in replacements {
                    let id = format!(
                        "arithmetic/{}_to_{}@{}:{}",
                        op, replacement, path.display(), begin
                    );

                    mutations.push(Mutation {
                        id,
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: format!("arithmetic/{}_to_{}", op, replacement),
                        original: original.clone(),
                        replacement: replacement.to_string(),
                        byte_range: begin..end,
                    });
                }
            }
        }
    }
}

/// Walk all child nodes of any Node variant.
///
/// lib-ruby-parser v4 does not expose a public children() method, so we
/// extract child nodes from each variant manually. This macro-style match
/// covers every variant that contains `Node`, `Option<Box<Node>>`, or
/// `Vec<Node>` fields.
fn visit_node(node: &Node, path: &PathBuf, source: &[u8], mutations: &mut Vec<Mutation>) {
    macro_rules! v {
        ($n:expr) => { visit_node($n, path, source, mutations) };
    }
    macro_rules! v_opt {
        ($n:expr) => { if let Some(ref inner) = $n { v!(inner); } };
    }
    macro_rules! v_vec {
        ($n:expr) => { for item in $n { v!(item); } };
    }

    match node {
        Node::Send(n) => {
            visit_send(n, path, source, mutations);
            v_opt!(n.recv);
            v_vec!(&n.args);
        }
        Node::CSend(n) => { v!(&n.recv); v_vec!(&n.args); }
        Node::Begin(n) => v_vec!(&n.statements),
        Node::Block(n) => { v!(&n.call); v_opt!(n.args); v_opt!(n.body); }
        Node::Def(n) => { v_opt!(n.args); v_opt!(n.body); }
        Node::Defs(n) => { v!(&n.definee); v_opt!(n.args); v_opt!(n.body); }
        Node::Class(n) => { v!(&n.name); v_opt!(n.superclass); v_opt!(n.body); }
        Node::Module(n) => { v!(&n.name); v_opt!(n.body); }
        Node::SClass(n) => { v!(&n.expr); v_opt!(n.body); }
        Node::If(n) => { v!(&n.cond); v_opt!(n.if_true); v_opt!(n.if_false); }
        Node::IfMod(n) => { v!(&n.cond); v_opt!(n.if_true); v_opt!(n.if_false); }
        Node::While(n) => { v!(&n.cond); v_opt!(n.body); }
        Node::WhilePost(n) => { v!(&n.cond); v!(&n.body); }
        Node::Until(n) => { v!(&n.cond); v_opt!(n.body); }
        Node::UntilPost(n) => { v!(&n.cond); v!(&n.body); }
        Node::For(n) => { v!(&n.iterator); v!(&n.iteratee); v_opt!(n.body); }
        Node::Case(n) => { v_opt!(n.expr); v_vec!(&n.when_bodies); v_opt!(n.else_body); }
        Node::When(n) => { v_vec!(&n.patterns); v_opt!(n.body); }
        Node::And(n) => { v!(&n.lhs); v!(&n.rhs); }
        Node::Or(n) => { v!(&n.lhs); v!(&n.rhs); }
        Node::AndAsgn(n) => { v!(&n.recv); v!(&n.value); }
        Node::OrAsgn(n) => { v!(&n.recv); v!(&n.value); }
        Node::OpAsgn(n) => { v!(&n.recv); v!(&n.value); }
        Node::Masgn(n) => { v!(&n.lhs); v!(&n.rhs); }
        Node::Lvasgn(n) => v_opt!(n.value),
        Node::Ivasgn(n) => v_opt!(n.value),
        Node::Cvasgn(n) => v_opt!(n.value),
        Node::Gvasgn(n) => v_opt!(n.value),
        Node::Casgn(n) => v_opt!(n.value),
        Node::Mlhs(n) => v_vec!(&n.items),
        Node::Array(n) => v_vec!(&n.elements),
        Node::Hash(n) => v_vec!(&n.pairs),
        Node::Pair(n) => { v!(&n.key); v!(&n.value); }
        Node::Return(n) => v_vec!(&n.args),
        Node::Yield(n) => v_vec!(&n.args),
        Node::Break(n) => v_vec!(&n.args),
        Node::Next(n) => v_vec!(&n.args),
        Node::Rescue(n) => { v_opt!(n.body); v_vec!(&n.rescue_bodies); v_opt!(n.else_); }
        Node::RescueBody(n) => { v_opt!(n.exc_list); v_opt!(n.exc_var); v_opt!(n.body); }
        Node::Ensure(n) => { v_opt!(n.body); v_opt!(n.ensure); }
        Node::KwBegin(n) => v_vec!(&n.statements),
        Node::Args(n) => v_vec!(&n.args),
        Node::Kwarg(_) => {}
        Node::Kwoptarg(n) => v!(&n.default),
        Node::Optarg(n) => v!(&n.default),
        Node::Splat(n) => v_opt!(n.value),
        Node::Kwsplat(n) => v!(&n.value),
        Node::Dstr(n) => v_vec!(&n.parts),
        Node::Dsym(n) => v_vec!(&n.parts),
        Node::Regexp(n) => v_vec!(&n.parts),
        Node::Index(n) => { v!(&n.recv); v_vec!(&n.indexes); }
        Node::IndexAsgn(n) => { v!(&n.recv); v_vec!(&n.indexes); v_opt!(n.value); }
        Node::Super(n) => v_vec!(&n.args),
        Node::Defined(n) => v!(&n.value),
        Node::MatchPattern(n) => { v!(&n.value); v!(&n.pattern); }
        Node::MatchPatternP(n) => { v!(&n.value); v!(&n.pattern); }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SourceFile;

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
