use super::Mutation;
use lib_ruby_parser::Node;
use std::path::PathBuf;

/// Get the expression byte range (begin, end) of any Node.
pub fn node_expression(node: &Node) -> (usize, usize) {
    macro_rules! expr {
        ($n:expr) => {
            ($n.expression_l.begin, $n.expression_l.end)
        };
    }

    match node {
        Node::Alias(n) => expr!(n),
        Node::And(n) => expr!(n),
        Node::AndAsgn(n) => expr!(n),
        Node::Arg(n) => expr!(n),
        Node::Args(n) => expr!(n),
        Node::Array(n) => expr!(n),
        Node::ArrayPattern(n) => expr!(n),
        Node::ArrayPatternWithTail(n) => expr!(n),
        Node::BackRef(n) => expr!(n),
        Node::Begin(n) => expr!(n),
        Node::Block(n) => expr!(n),
        Node::Blockarg(n) => expr!(n),
        Node::BlockPass(n) => expr!(n),
        Node::Break(n) => expr!(n),
        Node::Case(n) => expr!(n),
        Node::CaseMatch(n) => expr!(n),
        Node::Casgn(n) => expr!(n),
        Node::Cbase(n) => expr!(n),
        Node::Class(n) => expr!(n),
        Node::Complex(n) => expr!(n),
        Node::Const(n) => expr!(n),
        Node::ConstPattern(n) => expr!(n),
        Node::CSend(n) => expr!(n),
        Node::Cvar(n) => expr!(n),
        Node::Cvasgn(n) => expr!(n),
        Node::Def(n) => expr!(n),
        Node::Defined(n) => expr!(n),
        Node::Defs(n) => expr!(n),
        Node::Dstr(n) => expr!(n),
        Node::Dsym(n) => expr!(n),
        Node::EFlipFlop(n) => expr!(n),
        Node::EmptyElse(n) => expr!(n),
        Node::Encoding(n) => expr!(n),
        Node::Ensure(n) => expr!(n),
        Node::Erange(n) => expr!(n),
        Node::False(n) => expr!(n),
        Node::File(n) => expr!(n),
        Node::FindPattern(n) => expr!(n),
        Node::Float(n) => expr!(n),
        Node::For(n) => expr!(n),
        Node::ForwardArg(n) => expr!(n),
        Node::ForwardedArgs(n) => expr!(n),
        Node::Gvar(n) => expr!(n),
        Node::Gvasgn(n) => expr!(n),
        Node::Hash(n) => expr!(n),
        Node::HashPattern(n) => expr!(n),
        Node::Heredoc(n) => expr!(n),
        Node::If(n) => expr!(n),
        Node::IfGuard(n) => expr!(n),
        Node::IFlipFlop(n) => expr!(n),
        Node::IfMod(n) => expr!(n),
        Node::IfTernary(n) => expr!(n),
        Node::Index(n) => expr!(n),
        Node::IndexAsgn(n) => expr!(n),
        Node::InPattern(n) => expr!(n),
        Node::Int(n) => expr!(n),
        Node::Irange(n) => expr!(n),
        Node::Ivar(n) => expr!(n),
        Node::Ivasgn(n) => expr!(n),
        Node::Kwarg(n) => expr!(n),
        Node::KwBegin(n) => expr!(n),
        Node::Kwnilarg(n) => expr!(n),
        Node::Kwoptarg(n) => expr!(n),
        Node::Kwrestarg(n) => expr!(n),
        Node::Kwsplat(n) => expr!(n),
        Node::Lambda(n) => expr!(n),
        Node::Line(n) => expr!(n),
        Node::Lvar(n) => expr!(n),
        Node::Lvasgn(n) => expr!(n),
        Node::Masgn(n) => expr!(n),
        Node::MatchAlt(n) => expr!(n),
        Node::MatchAs(n) => expr!(n),
        Node::MatchCurrentLine(n) => expr!(n),
        Node::MatchNilPattern(n) => expr!(n),
        Node::MatchPattern(n) => expr!(n),
        Node::MatchPatternP(n) => expr!(n),
        Node::MatchRest(n) => expr!(n),
        Node::MatchVar(n) => expr!(n),
        Node::MatchWithLvasgn(n) => expr!(n),
        Node::Mlhs(n) => expr!(n),
        Node::Module(n) => expr!(n),
        Node::Next(n) => expr!(n),
        Node::Nil(n) => expr!(n),
        Node::NthRef(n) => expr!(n),
        Node::Numblock(n) => expr!(n),
        Node::OpAsgn(n) => expr!(n),
        Node::Optarg(n) => expr!(n),
        Node::Or(n) => expr!(n),
        Node::OrAsgn(n) => expr!(n),
        Node::Pair(n) => expr!(n),
        Node::Pin(n) => expr!(n),
        Node::Postexe(n) => expr!(n),
        Node::Preexe(n) => expr!(n),
        Node::Procarg0(n) => expr!(n),
        Node::Rational(n) => expr!(n),
        Node::Redo(n) => expr!(n),
        Node::Regexp(n) => expr!(n),
        Node::RegOpt(n) => expr!(n),
        Node::Rescue(n) => expr!(n),
        Node::RescueBody(n) => expr!(n),
        Node::Restarg(n) => expr!(n),
        Node::Retry(n) => expr!(n),
        Node::Return(n) => expr!(n),
        Node::SClass(n) => expr!(n),
        Node::Self_(n) => expr!(n),
        Node::Send(n) => expr!(n),
        Node::Shadowarg(n) => expr!(n),
        Node::Splat(n) => expr!(n),
        Node::Str(n) => expr!(n),
        Node::Super(n) => expr!(n),
        Node::Sym(n) => expr!(n),
        Node::True(n) => expr!(n),
        Node::Undef(n) => expr!(n),
        Node::UnlessGuard(n) => expr!(n),
        Node::Until(n) => expr!(n),
        Node::UntilPost(n) => expr!(n),
        Node::When(n) => expr!(n),
        Node::While(n) => expr!(n),
        Node::WhilePost(n) => expr!(n),
        Node::XHeredoc(n) => expr!(n),
        Node::Xstr(n) => expr!(n),
        Node::Yield(n) => expr!(n),
        Node::ZSuper(n) => expr!(n),
        Node::Kwargs(n) => expr!(n),
    }
}

/// Walk child nodes of any Node variant, calling a visitor function on each.
///
/// This is the shared tree-walking function used by all mutators. Each mutator
/// calls this to recurse into child nodes after handling the current node.
pub fn walk_children(
    node: &Node,
    path: &PathBuf,
    source: &[u8],
    _mutations: &mut Vec<Mutation>,
) {
    // This function intentionally does nothing — each mutator's visit_node
    // handles its own recursion by calling walk_node directly.
    // This is a no-op placeholder kept for backward compatibility.
    let _ = (node, path, source);
}

/// Generic tree walker that visits all nodes and calls a callback on each.
pub fn walk_all<F>(node: &Node, f: &mut F)
where
    F: FnMut(&Node),
{
    f(node);

    macro_rules! v {
        ($n:expr) => {
            walk_all($n, f)
        };
    }
    macro_rules! v_opt {
        ($n:expr) => {
            if let Some(ref inner) = $n {
                walk_all(inner, f);
            }
        };
    }
    macro_rules! v_vec {
        ($n:expr) => {
            for item in $n {
                walk_all(item, f);
            }
        };
    }

    match node {
        Node::Send(n) => {
            v_opt!(n.recv);
            v_vec!(&n.args);
        }
        Node::CSend(n) => {
            v!(&n.recv);
            v_vec!(&n.args);
        }
        Node::Begin(n) => v_vec!(&n.statements),
        Node::Block(n) => {
            v!(&n.call);
            v_opt!(n.args);
            v_opt!(n.body);
        }
        Node::Def(n) => {
            v_opt!(n.args);
            v_opt!(n.body);
        }
        Node::Defs(n) => {
            v!(&n.definee);
            v_opt!(n.args);
            v_opt!(n.body);
        }
        Node::Class(n) => {
            v!(&n.name);
            v_opt!(n.superclass);
            v_opt!(n.body);
        }
        Node::Module(n) => {
            v!(&n.name);
            v_opt!(n.body);
        }
        Node::SClass(n) => {
            v!(&n.expr);
            v_opt!(n.body);
        }
        Node::If(n) => {
            v!(&n.cond);
            v_opt!(n.if_true);
            v_opt!(n.if_false);
        }
        Node::IfMod(n) => {
            v!(&n.cond);
            v_opt!(n.if_true);
            v_opt!(n.if_false);
        }
        Node::IfTernary(n) => {
            v!(&n.cond);
            v!(&n.if_true);
            v!(&n.if_false);
        }
        Node::While(n) => {
            v!(&n.cond);
            v_opt!(n.body);
        }
        Node::WhilePost(n) => {
            v!(&n.cond);
            v!(&n.body);
        }
        Node::Until(n) => {
            v!(&n.cond);
            v_opt!(n.body);
        }
        Node::UntilPost(n) => {
            v!(&n.cond);
            v!(&n.body);
        }
        Node::For(n) => {
            v!(&n.iterator);
            v!(&n.iteratee);
            v_opt!(n.body);
        }
        Node::Case(n) => {
            v_opt!(n.expr);
            v_vec!(&n.when_bodies);
            v_opt!(n.else_body);
        }
        Node::When(n) => {
            v_vec!(&n.patterns);
            v_opt!(n.body);
        }
        Node::And(n) => {
            v!(&n.lhs);
            v!(&n.rhs);
        }
        Node::Or(n) => {
            v!(&n.lhs);
            v!(&n.rhs);
        }
        Node::AndAsgn(n) => {
            v!(&n.recv);
            v!(&n.value);
        }
        Node::OrAsgn(n) => {
            v!(&n.recv);
            v!(&n.value);
        }
        Node::OpAsgn(n) => {
            v!(&n.recv);
            v!(&n.value);
        }
        Node::Masgn(n) => {
            v!(&n.lhs);
            v!(&n.rhs);
        }
        Node::Lvasgn(n) => v_opt!(n.value),
        Node::Ivasgn(n) => v_opt!(n.value),
        Node::Cvasgn(n) => v_opt!(n.value),
        Node::Gvasgn(n) => v_opt!(n.value),
        Node::Casgn(n) => v_opt!(n.value),
        Node::Mlhs(n) => v_vec!(&n.items),
        Node::Array(n) => v_vec!(&n.elements),
        Node::Hash(n) => v_vec!(&n.pairs),
        Node::Pair(n) => {
            v!(&n.key);
            v!(&n.value);
        }
        Node::Return(n) => v_vec!(&n.args),
        Node::Yield(n) => v_vec!(&n.args),
        Node::Break(n) => v_vec!(&n.args),
        Node::Next(n) => v_vec!(&n.args),
        Node::Rescue(n) => {
            v_opt!(n.body);
            v_vec!(&n.rescue_bodies);
            v_opt!(n.else_);
        }
        Node::RescueBody(n) => {
            v_opt!(n.exc_list);
            v_opt!(n.exc_var);
            v_opt!(n.body);
        }
        Node::Ensure(n) => {
            v_opt!(n.body);
            v_opt!(n.ensure);
        }
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
        Node::Index(n) => {
            v!(&n.recv);
            v_vec!(&n.indexes);
        }
        Node::IndexAsgn(n) => {
            v!(&n.recv);
            v_vec!(&n.indexes);
            v_opt!(n.value);
        }
        Node::Super(n) => v_vec!(&n.args),
        Node::Defined(n) => v!(&n.value),
        Node::MatchPattern(n) => {
            v!(&n.value);
            v!(&n.pattern);
        }
        Node::MatchPatternP(n) => {
            v!(&n.value);
            v!(&n.pattern);
        }
        Node::Numblock(n) => {
            v!(&n.call);
            v!(&n.body);
        }
        Node::Postexe(n) => v_opt!(n.body),
        Node::Preexe(n) => v_opt!(n.body),
        _ => {}
    }
}
