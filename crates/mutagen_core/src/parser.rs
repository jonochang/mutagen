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
    /// Convert a byte offset into 1-based (line, col).
    pub fn byte_offset_to_line_col(&self, offset: usize) -> (u32, u32) {
        let mut line = 1u32;
        let mut col = 1u32;
        for &b in &self.source[..offset.min(self.source.len())] {
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

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
