use magnus::{function, prelude::*, Error, RArray, Ruby};

fn generate_mutations(ruby: &Ruby, source_code: String, file_path: String) -> Result<RArray, Error> {
    let source_file = mutagen_core::parser::SourceFile::parse(
        std::path::PathBuf::from(&file_path),
        source_code.into_bytes(),
    );

    let registry = mutagen_core::mutators::MutatorRegistry::default_registry();
    let mutations = registry.generate_all(&source_file);

    let results = ruby.ary_new_capa(mutations.len());

    for m in mutations {
        let hash = ruby.hash_new();
        hash.aset(ruby.to_symbol("id"), m.id.as_str())?;
        hash.aset(ruby.to_symbol("file"), m.file.to_string_lossy().as_ref())?;
        hash.aset(ruby.to_symbol("line"), m.line as i64)?;
        hash.aset(ruby.to_symbol("col"), m.col as i64)?;
        hash.aset(ruby.to_symbol("operator"), m.operator.as_str())?;
        hash.aset(ruby.to_symbol("original"), m.original.as_str())?;
        hash.aset(ruby.to_symbol("replacement"), m.replacement.as_str())?;
        hash.aset(ruby.to_symbol("byte_range_start"), m.byte_range.start as i64)?;
        hash.aset(ruby.to_symbol("byte_range_end"), m.byte_range.end as i64)?;
        results.push(hash)?;
    }

    Ok(results)
}

fn apply_mutation(
    _ruby: &Ruby,
    source_code: String,
    byte_range_start: usize,
    byte_range_end: usize,
    replacement: String,
) -> Result<String, Error> {
    let mutation = mutagen_core::mutators::Mutation {
        id: String::new(),
        file: std::path::PathBuf::new(),
        line: 0,
        col: 0,
        operator: String::new(),
        original: String::new(),
        replacement,
        byte_range: byte_range_start..byte_range_end,
    };

    Ok(mutagen_core::codegen::apply_mutation(
        source_code.as_bytes(),
        &mutation,
    ))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Mutagen")?;
    let native = module.define_module("Native")?;

    native.define_module_function("generate_mutations", function!(generate_mutations, 2))?;
    native.define_module_function("apply_mutation", function!(apply_mutation, 4))?;

    Ok(())
}
