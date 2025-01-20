use std::ops::Sub;
use std::time::{Duration, Instant};
use crate::cache::CacheEntry;

#[test]
fn test_cache_entry_serialization() {
    let entry = CacheEntry {
        timestamp: Instant::now().sub(Duration::from_secs(20)),
        value: "test".to_string()
    };

    let serialized = rocket::serde::json::to_string(&entry).unwrap();

    // wait for a while
    std::thread::sleep(Duration::from_secs(5));

    let deserialized: CacheEntry = rocket::serde::json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.timestamp.duration_since(entry.timestamp).as_secs(), 0);
}
