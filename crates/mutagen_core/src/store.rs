use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub status: String,
    pub killing_test: Option<String>,
    pub duration_ms: Option<u64>,
    pub source_hash: String,
    pub mutation_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
    pub version: u32,
    pub runs: HashMap<String, CacheEntry>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            version: 1,
            runs: HashMap::new(),
        }
    }

    pub fn insert(&mut self, mutation_id: String, entry: CacheEntry) {
        self.runs.insert(mutation_id, entry);
    }

    /// Check if a mutation has a cached result with matching source hash.
    pub fn check(&self, mutation_id: &str, source_hash: &str) -> Option<&CacheEntry> {
        self.runs.get(mutation_id).and_then(|entry| {
            if entry.source_hash == source_hash {
                Some(entry)
            } else {
                None
            }
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let cache: Cache = serde_json::from_str(&json)?;
        Ok(cache)
    }

    /// Merge another cache into this one. Entries from `other` overwrite
    /// entries in `self` if the source hash is newer (different).
    pub fn merge(&mut self, other: &Cache) {
        for (id, entry) in &other.runs {
            self.runs.insert(id.clone(), entry.clone());
        }
    }

    /// Merge multiple cache files into a single cache.
    pub fn merge_files(paths: &[&Path]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut merged = Cache::new();
        for path in paths {
            let cache = Cache::load(path)?;
            merged.merge(&cache);
        }
        Ok(merged)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn cache_hit_with_matching_hash() {
        let mut cache = Cache::new();
        cache.insert(
            "m1".to_string(),
            CacheEntry {
                status: "killed".to_string(),
                killing_test: None,
                duration_ms: None,
                source_hash: "abc123".to_string(),
                mutation_hash: "mut1".to_string(),
            },
        );
        let result = cache.check("m1", "abc123");
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, "killed");
    }

    #[test]
    fn cache_miss_with_different_hash() {
        let mut cache = Cache::new();
        cache.insert(
            "m1".to_string(),
            CacheEntry {
                status: "killed".to_string(),
                killing_test: None,
                duration_ms: None,
                source_hash: "abc123".to_string(),
                mutation_hash: "mut1".to_string(),
            },
        );
        let result = cache.check("m1", "def456");
        assert!(result.is_none());
    }

    #[test]
    fn cache_miss_for_unknown_mutation() {
        let cache = Cache::new();
        let result = cache.check("m1", "abc123");
        assert!(result.is_none());
    }

    #[test]
    fn round_trip_save_load() {
        let mut cache = Cache::new();
        cache.insert(
            "m1".to_string(),
            CacheEntry {
                status: "survived".to_string(),
                killing_test: None,
                duration_ms: None,
                source_hash: "abc123".to_string(),
                mutation_hash: "mut1".to_string(),
            },
        );

        let tmp = NamedTempFile::new().unwrap();
        cache.save(tmp.path()).unwrap();

        let loaded = Cache::load(tmp.path()).unwrap();
        let result = loaded.check("m1", "abc123");
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, "survived");
    }
}
