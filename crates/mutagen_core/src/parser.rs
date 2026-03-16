use lib_ruby_parser::{Parser, ParserOptions, ParserResult};
use std::path::PathBuf;

pub struct SourceFile {
    // Note: ParserResult does not implement Debug, so we use a manual impl below
    pub path: PathBuf,
    pub source: Vec<u8>,
    pub result: ParserResult,
}

impl std::fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceFile")
            .field("path", &self.path)
            .field("source_len", &self.source.len())
            .finish()
    }
}

impl SourceFile {
    pub fn parse(path: PathBuf, source: Vec<u8>) -> Self {
        let options = ParserOptions {
            buffer_name: path.to_string_lossy().to_string(),
            ..Default::default()
        };
        let parser = Parser::new(source.clone(), options);
        let result = parser.do_parse();
        Self {
            path,
            source,
            result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_ruby() {
        let source = b"1 + 2".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        assert!(sf.result.ast.is_some());
    }

    #[test]
    fn parses_variable_arithmetic() {
        let source = b"a + b".to_vec();
        let sf = SourceFile::parse(PathBuf::from("test.rb"), source);
        assert!(sf.result.ast.is_some());
    }
}
