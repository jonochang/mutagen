use cucumber::{given, then, when, World};
use mutagen_core::codegen;
use mutagen_core::mutators::{Mutation, MutatorRegistry};
use mutagen_core::parser::SourceFile;
use std::path::PathBuf;

#[derive(Debug, Default, World)]
struct MutationWorld {
    source: Option<SourceFile>,
    mutations: Vec<Mutation>,
    mutated_source: Option<String>,
}

#[given(expr = "a Ruby source {string}")]
fn ruby_source(world: &mut MutationWorld, source: String) {
    world.source = Some(SourceFile::parse(
        PathBuf::from("test.rb"),
        source.into_bytes(),
    ));
}

#[when("I generate mutations")]
fn generate_mutations(world: &mut MutationWorld) {
    let source = world.source.as_ref().expect("source must be set");
    let registry = MutatorRegistry::default_registry();
    world.mutations = registry.generate_all(source);
}

#[when(expr = "I apply mutation {int}")]
fn apply_mutation(world: &mut MutationWorld, index: usize) {
    let source = world.source.as_ref().expect("source must be set");
    let mutation = &world.mutations[index];
    world.mutated_source = Some(codegen::apply_mutation(&source.source, mutation));
}

#[then(expr = "I should see {int} mutation(s)")]
fn check_mutation_count(world: &mut MutationWorld, count: usize) {
    assert_eq!(
        world.mutations.len(),
        count,
        "expected {} mutations but got {}: {:?}",
        count,
        world.mutations.len(),
        world.mutations
    );
}

#[then(expr = "mutation {int} should replace {string} with {string}")]
fn check_mutation_replacement(
    world: &mut MutationWorld,
    index: usize,
    original: String,
    replacement: String,
) {
    let mutation = &world.mutations[index];
    assert_eq!(
        mutation.original, original,
        "expected original '{}' but got '{}'",
        original, mutation.original
    );
    assert_eq!(
        mutation.replacement, replacement,
        "expected replacement '{}' but got '{}'",
        replacement, mutation.replacement
    );
}

#[then(expr = "the mutated source should be {string}")]
fn check_mutated_source(world: &mut MutationWorld, expected: String) {
    let actual = world.mutated_source.as_ref().expect("mutated source must be set");
    assert_eq!(
        actual, &expected,
        "expected mutated source '{}' but got '{}'",
        expected, actual
    );
}

fn main() {
    futures::executor::block_on(
        MutationWorld::run("../../tests/features/mutations"),
    );
}
