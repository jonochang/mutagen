use crate::mutators::Mutation;

pub fn apply_mutation(source: &[u8], mutation: &Mutation) -> String {
    let mut result = Vec::with_capacity(source.len());
    result.extend_from_slice(&source[..mutation.byte_range.start]);
    result.extend_from_slice(mutation.replacement.as_bytes());
    result.extend_from_slice(&source[mutation.byte_range.end..]);
    String::from_utf8_lossy(&result).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutators::Mutation;
    use std::path::PathBuf;

    #[test]
    fn applies_simple_replacement() {
        let source = b"a + b";
        let mutation = Mutation {
            id: "test".to_string(),
            file: PathBuf::from("test.rb"),
            line: 1,
            col: 2,
            operator: "arithmetic/+_to_-".to_string(),
            original: "+".to_string(),
            replacement: "-".to_string(),
            byte_range: 2..3,
        };

        let result = apply_mutation(source, &mutation);
        assert_eq!(result, "a - b");
    }
}
