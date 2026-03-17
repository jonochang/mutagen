use cucumber::{given, then, when, World};
use mutagen_core::mutators::Mutation;
use mutagen_core::selector;
use mutagen_core::store::{Cache, CacheEntry};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Default, World)]
struct SelectorWorld {
    mutations: Vec<Mutation>,
    coverage: HashMap<PathBuf, HashSet<u32>>,
    filtered: Vec<Mutation>,
    cache: Cache,
    cache_result: Option<String>,
    temp_path: Option<PathBuf>,
}

fn make_mutation(file: &str, line: u32) -> Mutation {
    Mutation {
        id: format!("test@{}:{}", file, line),
        file: PathBuf::from(file),
        line,
        col: 0,
        operator: "test".to_string(),
        original: "x".to_string(),
        replacement: "y".to_string(),
        byte_range: 0..1,
    }
}

// ---- Coverage steps ----

#[given(expr = "mutations at lines {int}, {int}, {int} in {string}")]
fn mutations_at_lines(world: &mut SelectorWorld, l1: u32, l2: u32, l3: u32, file: String) {
    world.mutations = vec![
        make_mutation(&file, l1),
        make_mutation(&file, l2),
        make_mutation(&file, l3),
    ];
}

#[given(expr = "coverage data covering lines {int}, {int} in {string}")]
fn coverage_for_lines(world: &mut SelectorWorld, l1: u32, l2: u32, file: String) {
    world
        .coverage
        .insert(PathBuf::from(file), HashSet::from([l1, l2]));
}

#[given("no coverage data")]
fn no_coverage(world: &mut SelectorWorld) {
    world.coverage.clear();
}

#[when("I filter by coverage")]
fn filter_coverage(world: &mut SelectorWorld) {
    let mutations = std::mem::take(&mut world.mutations);
    world.filtered = selector::filter_by_coverage(mutations, &world.coverage);
}

#[then(expr = "I should have {int} remaining mutations")]
fn check_remaining_count(world: &mut SelectorWorld, count: usize) {
    assert_eq!(
        world.filtered.len(),
        count,
        "expected {} remaining but got {}",
        count,
        world.filtered.len()
    );
}

#[then(expr = "the remaining mutations should be at lines {int}, {int}")]
fn check_remaining_lines(world: &mut SelectorWorld, l1: u32, l2: u32) {
    let lines: Vec<u32> = world.filtered.iter().map(|m| m.line).collect();
    assert!(
        lines.contains(&l1) && lines.contains(&l2),
        "expected lines {:?} to contain {} and {}",
        lines,
        l1,
        l2
    );
}

// ---- Sampling steps ----

#[given(expr = "{int} mutations")]
fn n_mutations(world: &mut SelectorWorld, count: usize) {
    world.mutations = (0..count as u32).map(|i| make_mutation("app.rb", i)).collect();
}

#[when(expr = "I sample {int} mutations")]
fn sample_count(world: &mut SelectorWorld, count: usize) {
    let mutations = std::mem::take(&mut world.mutations);
    world.filtered = selector::sample_count(mutations, count);
}

#[when(expr = "I sample {int} percent")]
fn sample_percent(world: &mut SelectorWorld, percent: u32) {
    let mutations = std::mem::take(&mut world.mutations);
    world.filtered = selector::sample_percent(mutations, percent);
}

// ---- Sharding steps ----

#[when(expr = "I shard into {int} with index {int}")]
fn shard_mutations(world: &mut SelectorWorld, total: u32, index: u32) {
    let mutations = std::mem::take(&mut world.mutations);
    world.filtered = selector::shard(mutations, index, total);
}

#[when(expr = "I collect all {int} shards")]
fn collect_all_shards(world: &mut SelectorWorld, total: u32) {
    let mutations = world.mutations.clone();
    let mut all = Vec::new();
    for i in 1..=total {
        all.extend(selector::shard(mutations.clone(), i, total));
    }
    world.filtered = all;
}

#[then(expr = "I should have between {int} and {int} remaining mutations")]
fn check_remaining_range(world: &mut SelectorWorld, min: usize, max: usize) {
    let len = world.filtered.len();
    assert!(
        len >= min && len <= max,
        "expected between {} and {} remaining but got {}",
        min,
        max,
        len
    );
}

#[then(expr = "all {int} mutations should be covered")]
fn check_all_covered(world: &mut SelectorWorld, expected: usize) {
    let mut ids: Vec<String> = world.filtered.iter().map(|m| m.id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        expected,
        "expected {} unique mutations but got {}",
        expected,
        ids.len()
    );
}

// ---- Cache steps ----

#[given(expr = "a cached result for mutation {string} with status {string} and source hash {string}")]
fn cached_result(world: &mut SelectorWorld, id: String, status: String, hash: String) {
    world.cache.insert(
        id,
        CacheEntry {
            status,
            killing_test: None,
            duration_ms: None,
            source_hash: hash,
            mutation_hash: "mut".to_string(),
        },
    );
}

#[given("an empty cache")]
fn empty_cache(world: &mut SelectorWorld) {
    world.cache = Cache::new();
}

#[when(expr = "I check cache for mutation {string} with source hash {string}")]
fn check_cache(world: &mut SelectorWorld, id: String, hash: String) {
    world.cache_result = world.cache.check(&id, &hash).map(|e| e.status.clone());
}

#[when("I save the cache to a file")]
fn save_cache(world: &mut SelectorWorld) {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    world.cache.save(&path).unwrap();
    world.temp_path = Some(path);
    // Keep the temp file alive by leaking it (it's a test)
    std::mem::forget(tmp);
}

#[when("I load the cache from the file")]
fn load_cache(world: &mut SelectorWorld) {
    let path = world.temp_path.as_ref().unwrap();
    world.cache = Cache::load(path).unwrap();
}

#[then(expr = "the cache should return status {string}")]
fn cache_returns_status(world: &mut SelectorWorld, expected: String) {
    assert_eq!(
        world.cache_result.as_ref(),
        Some(&expected),
        "expected cache status '{}' but got {:?}",
        expected,
        world.cache_result
    );
}

#[then("the cache should return no result")]
fn cache_returns_none(world: &mut SelectorWorld) {
    assert!(
        world.cache_result.is_none(),
        "expected no cache result but got {:?}",
        world.cache_result
    );
}

fn main() {
    futures::executor::block_on(SelectorWorld::run("../../tests/features/selector"));
}
