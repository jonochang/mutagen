use crate::mutators::{Mutation, MutatorRegistry};
use crate::parser::SourceFile;
use rayon::prelude::*;
use std::path::PathBuf;

/// Process multiple files in parallel, generating mutations for each.
/// Returns (mutations, errors) where errors are (file_path, error_message) pairs.
pub fn generate_mutations_batch(
    files: Vec<(PathBuf, Vec<u8>)>,
    registry: &MutatorRegistry,
) -> (Vec<Mutation>, Vec<(PathBuf, String)>) {
    let results: Vec<_> = files
        .into_par_iter()
        .map(|(path, source)| {
            let source_file = SourceFile::parse(path.clone(), source);
            if source_file.result.diagnostics.iter().any(|d| d.is_error()) {
                let errors: Vec<String> = source_file
                    .result
                    .diagnostics
                    .iter()
                    .filter(|d| d.is_error())
                    .map(|d| d.render_message())
                    .collect();
                return Err((path, errors.join("; ")));
            }
            Ok(registry.generate_all(&source_file))
        })
        .collect();

    let mut mutations = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(muts) => mutations.extend(muts),
            Err(e) => errors.push(e),
        }
    }

    (mutations, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_multiple_files() {
        let registry = MutatorRegistry::default_registry();
        let files = vec![
            (PathBuf::from("a.rb"), b"x = 1 + 2".to_vec()),
            (PathBuf::from("b.rb"), b"y = 3 - 4".to_vec()),
        ];
        let (mutations, errors) = generate_mutations_batch(files, &registry);
        assert!(errors.is_empty());
        assert!(mutations.len() >= 2); // at least one arithmetic mutation per file
    }

    #[test]
    fn handles_empty_files() {
        let registry = MutatorRegistry::default_registry();
        let files = vec![(PathBuf::from("empty.rb"), b"".to_vec())];
        let (mutations, errors) = generate_mutations_batch(files, &registry);
        assert!(errors.is_empty());
        assert!(mutations.is_empty());
    }
}
