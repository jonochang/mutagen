use crate::mutators::Mutation;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Filter mutations to only those on lines covered by the test suite.
pub fn filter_by_coverage(
    mutations: Vec<Mutation>,
    coverage: &HashMap<PathBuf, HashSet<u32>>,
) -> Vec<Mutation> {
    if coverage.is_empty() {
        return mutations;
    }
    mutations
        .into_iter()
        .filter(|m| {
            coverage
                .get(&m.file)
                .map(|lines| lines.contains(&m.line))
                .unwrap_or(false)
        })
        .collect()
}

/// Sample a fixed number of mutations.
pub fn sample_count(mutations: Vec<Mutation>, count: usize) -> Vec<Mutation> {
    if count >= mutations.len() {
        return mutations;
    }
    // Deterministic sampling: take evenly spaced indices
    let step = mutations.len() as f64 / count as f64;
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let idx = (i as f64 * step) as usize;
        result.push(mutations[idx].clone());
    }
    result
}

/// Sample a percentage of mutations.
pub fn sample_percent(mutations: Vec<Mutation>, percent: u32) -> Vec<Mutation> {
    if percent >= 100 {
        return mutations;
    }
    let count = (mutations.len() as f64 * percent as f64 / 100.0).round() as usize;
    sample_count(mutations, count)
}

/// Shard mutations by stable hash. Returns shard `index` of `total` shards.
/// `index` is 1-based (1..=total).
pub fn shard(mutations: Vec<Mutation>, index: u32, total: u32) -> Vec<Mutation> {
    if total <= 1 {
        return mutations;
    }
    mutations
        .into_iter()
        .filter(|m| {
            let mut hasher = DefaultHasher::new();
            m.id.hash(&mut hasher);
            (hasher.finish() % total as u64) == (index as u64 - 1)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutators::Mutation;

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

    #[test]
    fn coverage_filter_keeps_covered_lines() {
        let mutations = vec![
            make_mutation("app.rb", 1),
            make_mutation("app.rb", 3),
            make_mutation("app.rb", 5),
        ];
        let mut coverage = HashMap::new();
        coverage.insert(
            PathBuf::from("app.rb"),
            HashSet::from([1, 5]),
        );
        let filtered = filter_by_coverage(mutations, &coverage);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].line, 1);
        assert_eq!(filtered[1].line, 5);
    }

    #[test]
    fn empty_coverage_keeps_all() {
        let mutations = vec![
            make_mutation("app.rb", 1),
            make_mutation("app.rb", 3),
        ];
        let coverage = HashMap::new();
        let filtered = filter_by_coverage(mutations, &coverage);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn sample_count_selects_exact() {
        let mutations: Vec<_> = (0..100).map(|i| make_mutation("app.rb", i)).collect();
        let sampled = sample_count(mutations, 25);
        assert_eq!(sampled.len(), 25);
    }

    #[test]
    fn sample_percent_selects_correctly() {
        let mutations: Vec<_> = (0..100).map(|i| make_mutation("app.rb", i)).collect();
        let sampled = sample_percent(mutations, 50);
        assert_eq!(sampled.len(), 50);
    }

    #[test]
    fn sample_100_percent_keeps_all() {
        let mutations: Vec<_> = (0..100).map(|i| make_mutation("app.rb", i)).collect();
        let sampled = sample_percent(mutations, 100);
        assert_eq!(sampled.len(), 100);
    }

    #[test]
    fn shard_distributes_all_mutations() {
        let mutations: Vec<_> = (0..100).map(|i| make_mutation("app.rb", i)).collect();
        let total_shards = 4;
        let mut all_ids: Vec<String> = Vec::new();
        for s in 1..=total_shards {
            let shard_mutations = shard(mutations.clone(), s, total_shards);
            all_ids.extend(shard_mutations.into_iter().map(|m| m.id));
        }
        all_ids.sort();
        let mut expected: Vec<String> = (0..100).map(|i| format!("test@app.rb:{}", i)).collect();
        expected.sort();
        assert_eq!(all_ids, expected);
    }

    #[test]
    fn shard_single_returns_all() {
        let mutations: Vec<_> = (0..10).map(|i| make_mutation("app.rb", i)).collect();
        let result = shard(mutations.clone(), 1, 1);
        assert_eq!(result.len(), 10);
    }
}
