use crate::mutators::{walk, Mutation, Mutator};
use crate::parser::SourceFile;
use lib_ruby_parser::Node;

pub struct RegexMutator;

impl Mutator for RegexMutator {
    fn category(&self) -> &str {
        "regex"
    }

    fn name(&self) -> &str {
        "regex"
    }

    fn generate(&self, source: &SourceFile) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        if let Some(ref ast) = source.result.ast {
            let path = &source.path;
            let src = &source.source;
            walk::walk_all(ast, &mut |node| {
                if let Node::Regexp(n) = node {
                    let content_begin = n.begin_l.end;
                    let content_end = n.end_l.begin;

                    if content_begin >= content_end {
                        return; // empty regex
                    }

                    let content =
                        String::from_utf8_lossy(&src[content_begin..content_end])
                            .to_string();

                    // Non-empty regex → empty (always matches)
                    mutations.push(Mutation {
                        id: format!(
                            "regex/to_empty@{}:{}",
                            path.display(),
                            content_begin
                        ),
                        file: path.clone(),
                        line: 0,
                        col: 0,
                        operator: "regex/to_empty".to_string(),
                        original: content.clone(),
                        replacement: String::new(),
                        byte_range: content_begin..content_end,
                    });

                    // \d → \w
                    if content.contains("\\d") {
                        mutations.push(Mutation {
                            id: format!(
                                "regex/d_to_w@{}:{}",
                                path.display(),
                                content_begin
                            ),
                            file: path.clone(),
                            line: 0,
                            col: 0,
                            operator: "regex/\\d_to_\\w".to_string(),
                            original: content.clone(),
                            replacement: content.replace("\\d", "\\w"),
                            byte_range: content_begin..content_end,
                        });
                    }

                    // + → *
                    if content.contains('+') {
                        mutations.push(Mutation {
                            id: format!(
                                "regex/plus_to_star@{}:{}",
                                path.display(),
                                content_begin
                            ),
                            file: path.clone(),
                            line: 0,
                            col: 0,
                            operator: "regex/+_to_*".to_string(),
                            original: content.clone(),
                            replacement: content.replace('+', "*"),
                            byte_range: content_begin..content_end,
                        });
                    }

                    // Remove ^ anchor
                    if content.contains('^') {
                        mutations.push(Mutation {
                            id: format!(
                                "regex/remove_caret@{}:{}",
                                path.display(),
                                content_begin
                            ),
                            file: path.clone(),
                            line: 0,
                            col: 0,
                            operator: "regex/remove_^".to_string(),
                            original: content.clone(),
                            replacement: content.replace('^', ""),
                            byte_range: content_begin..content_end,
                        });
                    }

                    // Remove $ anchor
                    if content.contains('$') {
                        mutations.push(Mutation {
                            id: format!(
                                "regex/remove_dollar@{}:{}",
                                path.display(),
                                content_begin
                            ),
                            file: path.clone(),
                            line: 0,
                            col: 0,
                            operator: "regex/remove_$".to_string(),
                            original: content.clone(),
                            replacement: content.replace('$', ""),
                            byte_range: content_begin..content_end,
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
    fn mutates_regex_to_empty() {
        let source = b"/foo/".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = RegexMutator.generate(&sf);

        let empty = mutations
            .iter()
            .find(|m| m.operator == "regex/to_empty");
        assert!(empty.is_some());
        assert_eq!(empty.unwrap().original, "foo");
        assert_eq!(empty.unwrap().replacement, "");
    }

    #[test]
    fn skips_empty_regex() {
        let source = b"//".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = RegexMutator.generate(&sf);

        assert!(mutations.is_empty());
    }

    #[test]
    fn mutates_digit_class() {
        let source = b"/\\d+/".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = RegexMutator.generate(&sf);

        let d_to_w = mutations
            .iter()
            .find(|m| m.operator == "regex/\\d_to_\\w");
        assert!(d_to_w.is_some());
    }

    #[test]
    fn mutates_plus_quantifier() {
        let source = b"/a+/".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = RegexMutator.generate(&sf);

        let plus = mutations
            .iter()
            .find(|m| m.operator == "regex/+_to_*");
        assert!(plus.is_some());
        assert_eq!(plus.unwrap().replacement, "a*");
    }

    #[test]
    fn removes_caret_anchor() {
        let source = b"/^foo/".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = RegexMutator.generate(&sf);

        let caret = mutations
            .iter()
            .find(|m| m.operator == "regex/remove_^");
        assert!(caret.is_some());
        assert_eq!(caret.unwrap().replacement, "foo");
    }

    #[test]
    fn removes_dollar_anchor() {
        let source = b"/foo$/".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = RegexMutator.generate(&sf);

        let dollar = mutations
            .iter()
            .find(|m| m.operator == "regex/remove_$");
        assert!(dollar.is_some());
        assert_eq!(dollar.unwrap().replacement, "foo");
    }

    #[test]
    fn handles_regex_with_flags() {
        let source = b"/foo/i".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        let mutations = RegexMutator.generate(&sf);

        let empty = mutations
            .iter()
            .find(|m| m.operator == "regex/to_empty");
        assert!(empty.is_some());
        assert_eq!(empty.unwrap().original, "foo");
    }
}
