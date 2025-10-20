//! Tests for the common module
use chrono::Datelike;
use trash_utilities::common::*;

#[test]
pub fn test_create_ahash_map() {
    let map: ahash::AHashMap<String, i32> = crypto::create_ahash_map();
    assert!(map.is_empty());
}

#[test]
pub fn test_wrap_bytes() {
    let data = b"Hello, world!";
    let bytes = crypto::wrap_bytes(data);
    assert_eq!(bytes.as_ref(), data);
}

#[test]
pub fn test_encode_base64() {
    let data = b"Hello, world!";
    let encoded = crypto::encode_base64(data);
    assert!(!encoded.is_empty());
    // Should not contain padding for this input
    assert!(!encoded.ends_with('='));
}

#[test]
pub fn test_decode_base64() {
    let data = b"Hello, world!";
    let encoded = crypto::encode_base64(data);
    let decoded = crypto::decode_base64(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
pub fn test_compress_brotli() {
    let data = b"This is some test data for compression that should be compressible.";
    let compressed = crypto::compress_brotli(data, 6).unwrap();
    assert!(!compressed.is_empty());
    // Compressed data should be smaller for compressible data
    assert!(compressed.len() < data.len());
}

#[test]
pub fn test_decompress_brotli() {
    let data = b"This is some test data for compression.";
    let compressed = crypto::compress_brotli(data, 6).unwrap();
    let decompressed = crypto::decompress_brotli(&compressed).unwrap();
    assert_eq!(decompressed, data);
}

#[test]
pub fn test_fast_hash() {
    let hash1 = crypto::fast_hash(b"hello");
    let hash2 = crypto::fast_hash(b"world");
    let hash1_again = crypto::fast_hash(b"hello");

    assert_ne!(hash1, hash2);
    assert_eq!(hash1, hash1_again); // Deterministic
    assert_ne!(hash1, 0);
}

#[test]
pub fn test_keyed_hash() {
    let key1 = b"key1";
    let key2 = b"key2";
    let data = b"test data";

    let hash1 = crypto::keyed_hash(key1, data);
    let hash2 = crypto::keyed_hash(key2, data);
    let hash1_again = crypto::keyed_hash(key1, data);

    assert_ne!(hash1, hash2); // Different keys produce different hashes
    assert_eq!(hash1, hash1_again); // Same key+data produces same hash
}

#[test]
pub fn test_verify_keyed_hash() {
    let key = b"test_key";
    let data = b"test_data";
    let hash = crypto::keyed_hash(key, data);

    assert!(crypto::verify_keyed_hash(key, data, hash));
    assert!(!crypto::verify_keyed_hash(b"wrong_key", data, hash));
    assert!(!crypto::verify_keyed_hash(key, b"wrong_data", hash));
    assert!(!crypto::verify_keyed_hash(key, data, hash + 1));
}

#[test]
pub fn test_parse_json_value() {
    let json = r#"{"name":"Alice","age":30,"active":true}"#;
    let parsed: serde_json::Value = json::parse_json_value(json).unwrap();

    assert_eq!(parsed["name"], "Alice");
    assert_eq!(parsed["age"], 30);
    assert_eq!(parsed["active"], true);
}

#[test]
pub fn test_to_json_value() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
    };

    let json = json::to_json_value(&test).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("42"));
}

#[test]
pub fn test_pretty_json_value() {
    let data = vec![1, 2, 3];
    let pretty = json::pretty_json_value(&data).unwrap();
    assert!(pretty.contains('\n')); // Should have newlines for pretty printing
    assert!(pretty.contains('['));
    assert!(pretty.contains(']'));
}

#[test]
pub fn test_validate_json() {
    assert!(json::validate_json(r#"{"valid": "json"}"#));
    assert!(json::validate_json("[1, 2, 3]"));
    assert!(json::validate_json(r#""string""#));
    assert!(json::validate_json("42"));

    assert!(!json::validate_json(r#"{"invalid": json}"#));
    assert!(!json::validate_json("not json"));
    assert!(!json::validate_json(
        r#"{"missing": "comma" "invalid": "json"}"#
    ));
}

#[test]
pub fn test_merge_json() {
    let base = r#"{"name":"Alice","age":30}"#;
    let extra = r#"{"city":"New York","job":"Engineer"}"#;
    let merged = json::merge_json(base, extra).unwrap();

    // Parse merged JSON to verify contents
    let parsed: serde_json::Value = serde_json::from_str(&merged).unwrap();
    assert_eq!(parsed["name"], "Alice");
    assert_eq!(parsed["age"], 30);
    assert_eq!(parsed["city"], "New York");
    assert_eq!(parsed["job"], "Engineer");
}

#[test]
pub fn test_extract_json_path() {
    let json = r#"{
        "user": {
            "name": "Alice",
            "profile": {
                "age": 30,
                "city": "Boston"
            }
        },
        "active": true
    }"#;

    let name = json::extract_json_path(json, "user.name").unwrap().unwrap();
    assert_eq!(name.as_str().unwrap(), "Alice");

    let age = json::extract_json_path(json, "user.profile.age")
        .unwrap()
        .unwrap();
    assert_eq!(age.as_u64().unwrap(), 30);

    let active = json::extract_json_path(json, "active").unwrap().unwrap();
    assert!(active.as_bool().unwrap());

    let missing = json::extract_json_path(json, "user.missing").unwrap();
    assert!(missing.is_none());

    let invalid_path = json::extract_json_path(json, "user.name.invalid");
    assert!(invalid_path.is_err());
}

#[test]
pub fn test_current_utc_time() {
    let now = utils::current_utc_time();
    assert!(now.timestamp() > 0); // Should be a valid timestamp
}

#[test]
pub fn test_format_datetime() {
    let dt = utils::current_utc_time();
    let formatted = utils::format_datetime(&dt);
    assert!(formatted.contains('T')); // RFC3339 format has T
    assert!(formatted.ends_with('Z')); // UTC timezone
}

#[test]
pub fn test_parse_datetime() {
    let input = "2023-01-01T12:00:00Z";
    let dt = utils::parse_datetime(input).unwrap();
    assert_eq!(dt.year(), 2023);
    assert_eq!(dt.month(), 1);
    assert_eq!(dt.day(), 1);
}

#[test]
pub fn test_parse_date() {
    let date = utils::parse_date("2023-12-25").unwrap();
    assert_eq!(date.year(), 2023);
    assert_eq!(date.month(), 12);
    assert_eq!(date.day(), 25);
}

#[test]
pub fn test_atomic_counter() {
    let counter = utils::AtomicCounter::new();
    assert_eq!(counter.get(), 0);

    assert_eq!(counter.increment(), 1);
    assert_eq!(counter.get(), 1);

    counter.reset();
    assert_eq!(counter.get(), 0);
}

#[test]
pub fn test_string_interner() {
    let interner = utils::StringInterner::new();
    assert_eq!(interner.len(), 0);
    assert!(interner.is_empty());

    let s1 = interner.intern("hello");
    let s2 = interner.intern("hello");
    assert_eq!(interner.len(), 1);
    assert_eq!(s1.as_ptr(), s2.as_ptr()); // Same memory location

    let s3 = interner.intern("world");
    assert_eq!(interner.len(), 2);
    assert_ne!(s1.as_ptr(), s3.as_ptr());

    assert!(!interner.is_empty());
}

#[test]
pub fn test_lru_cache() {
    let cache = utils::LruCache::new(3);
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());

    cache.insert("key1", "value1");
    cache.insert("key2", "value2");
    cache.insert("key3", "value3");

    assert_eq!(cache.len(), 3);
    assert_eq!(cache.get(&"key1"), Some("value1"));

    // Insert fourth item, should evict oldest (key1)
    cache.insert("key4", "value4");
    assert_eq!(cache.len(), 3);
    assert_eq!(cache.get(&"key1"), None);
    assert_eq!(cache.get(&"key4"), Some("value4"));

    assert!(!cache.is_empty());
}

#[test]
pub fn test_common() {
    test_create_ahash_map();
    test_wrap_bytes();
    test_encode_base64();
    test_decode_base64();
    test_compress_brotli();
    test_decompress_brotli();
    test_fast_hash();
    test_keyed_hash();
    test_verify_keyed_hash();
    test_parse_json_value();
    test_to_json_value();
    test_pretty_json_value();
    test_validate_json();
    test_merge_json();
    test_extract_json_path();
    test_current_utc_time();
    test_format_datetime();
    test_parse_datetime();
    test_parse_date();
    test_atomic_counter();
    test_string_interner();
    test_lru_cache();
}
