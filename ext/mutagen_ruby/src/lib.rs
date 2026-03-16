use magnus::{define_module, function, prelude::*, Error, Ruby};

fn generate_mutations(
    _ruby: &Ruby,
    source_code: String,
    file_path: String,
) -> Result<Vec<magnus::Value>, Error> {
    let source_file = mutagen_core::parser::SourceFile::parse(
        std::path::PathBuf::from(&file_path),
        source_code.into_bytes(),
    );

    let registry = mutagen_core::mutators::MutatorRegistry::default_registry();
    let mutations = registry.generate_all(&source_file);

    let ruby = unsafe { Ruby::get_unchecked() };
    let mut results = Vec::new();

    for m in mutations {
        let hash = ruby.hash_new();
        hash.aset(ruby.sym_new("id"), ruby.str_new(&m.id))?;
        hash.aset(ruby.sym_new("file"), ruby.str_new(&m.file.to_string_lossy()))?;
        hash.aset(ruby.sym_new("line"), m.line)?;
        hash.aset(ruby.sym_new("col"), m.col)?;
        hash.aset(ruby.sym_new("operator"), ruby.str_new(&m.operator))?;
        hash.aset(ruby.sym_new("original"), ruby.str_new(&m.original))?;
        hash.aset(ruby.sym_new("replacement"), ruby.str_new(&m.replacement))?;
        hash.aset(ruby.sym_new("byte_range_start"), m.byte_range.start)?;
        hash.aset(ruby.sym_new("byte_range_end"), m.byte_range.end)?;
        results.push(hash.as_value());
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
    let module = define_module("Mutagen")?;
    let native = module.define_module("Native")?;

    native.define_module_function("generate_mutations", function!(generate_mutations, 2))?;
    native.define_module_function("apply_mutation", function!(apply_mutation, 4))?;

    Ok(())
}
