//! Tests for the chars module
use trash_parallelism::chars::*;

#[test]
pub fn test_find_byte() {
    let data = b"hello world";
    assert_eq!(core::find_byte(data, b'w'), Some(6));
    assert_eq!(core::find_byte(data, b'z'), None);
}

#[test]
pub fn test_find_byte_all() {
    let data = b"hello world";
    let positions = core::find_byte_all(data, b'l');
    assert_eq!(positions, vec![2, 3, 9]);
}

#[test]
pub fn test_find_any_byte() {
    let data = b"hello world";
    let separators = b" ,!";
    assert_eq!(core::find_any_byte(data, separators), Some(5)); // space at position 5
}

#[test]
pub fn test_hash_string_fast() {
    let hash1 = core::hash_string_fast("hello");
    let hash2 = core::hash_string_fast("world");
    let hash1_again = core::hash_string_fast("hello");

    assert_ne!(hash1, hash2);
    assert_eq!(hash1, hash1_again); // Deterministic
    assert_ne!(hash1, 0);
}

#[test]
pub fn test_encode_string_base64() {
    let encoded = core::encode_string_base64("hello world");
    assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
}

#[test]
pub fn test_decode_string_base64() {
    let decoded = core::decode_string_base64("aGVsbG8gd29ybGQ=").unwrap();
    assert_eq!(decoded, "hello world");
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_decode_string_base64_invalid() {
    let _ = core::decode_string_base64("invalid base64!").unwrap();
}

#[test]
pub fn test_hash_and_encode_base64() {
    let encoded_hash = core::hash_and_encode_base64("test data");
    assert!(!encoded_hash.is_empty());
    // Should be base64 encoded
    assert!(
        encoded_hash
            .chars()
            .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
    );
}

#[test]
pub fn test_split_string_efficient() {
    let parts = core::split_string_efficient("apple,banana,cherry", ',');
    assert_eq!(parts, vec!["apple", "banana", "cherry"]);

    let parts_empty = core::split_string_efficient("", ',');
    assert!(parts_empty.is_empty());

    let parts_no_delim = core::split_string_efficient("hello", ',');
    assert_eq!(parts_no_delim, vec!["hello"]);
}

#[test]
pub fn test_count_substring() {
    let count = core::count_substring("hello hello world", "hello");
    assert_eq!(count, 2);

    let count_zero = core::count_substring("test", "xyz");
    assert_eq!(count_zero, 0);

    let count_empty = core::count_substring("test", "");
    assert_eq!(count_empty, 0);
}

#[test]
pub fn test_deduplicate_lines() {
    let input = "line1\nline2\nline1\nline3\nline2";
    let result = core::deduplicate_lines(input);
    assert_eq!(result, "line1\nline2\nline3\n");

    let input_no_dupes = "a\nb\nc";
    let result_no_dupes = core::deduplicate_lines(input_no_dupes);
    assert_eq!(result_no_dupes, "a\nb\nc\n");

    let input_empty = "";
    let result_empty = core::deduplicate_lines(input_empty);
    assert_eq!(result_empty, "");
}

#[test]
pub fn test_string_interner() {
    let interner = core::StringInterner::new();

    let s1 = interner.intern("hello");
    let s2 = interner.intern("hello");
    assert_eq!(s1.as_ptr(), s2.as_ptr()); // Same memory location

    let s3 = interner.intern("world");
    assert_ne!(s1.as_ptr(), s3.as_ptr());

    let (count, bytes) = interner.stats();
    assert_eq!(count, 2); // Two unique strings
    assert!(bytes >= 9); // "hello" + "world" = 9 bytes
}

#[test]
pub fn test_efficient_string_builder() {
    let mut builder = core::EfficientStringBuilder::new();
    assert_eq!(builder.len(), 0);
    assert!(builder.is_empty());

    builder.append("Hello");
    builder.append(", ");
    builder.append("world!");
    assert_eq!(builder.len(), 13);
    assert!(!builder.is_empty());

    let result = builder.build();
    assert_eq!(result, "Hello, world!");

    // Test with capacity
    let mut builder_cap = core::EfficientStringBuilder::with_capacity(100);
    builder_cap.append_char('A');
    builder_cap.append_char('B');
    builder_cap.append_char('C');
    assert_eq!(builder_cap.build(), "ABC");
}

#[test]
pub fn test_parse_and_validate_json() {
    use serde::Deserialize;

    #[derive(Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let json = r#"{"name":"test","value":42}"#;
    let parsed: TestStruct = processing::parse_and_validate_json(json).unwrap();
    assert_eq!(
        parsed,
        TestStruct {
            name: "test".to_string(),
            value: 42
        }
    );
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_parse_and_validate_json_invalid() {
    let _ = processing::parse_and_validate_json::<serde_json::Value>("invalid json").unwrap();
}

#[test]
pub fn test_parallel_process_string() {
    let text = "The quick brown fox jumps over the lazy dog";
    let results = processing::parallel_process_string(text, 10, str::len);

    assert!(!results.is_empty());
    let total_len: usize = results.iter().sum();
    assert_eq!(total_len, text.len());
}

#[test]
pub fn test_extract_json_values_by_key() {
    let json = r#"[{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}, {"name": "Charlie", "age": 35}]"#;

    let names = processing::extract_json_values_by_key(json, "name").unwrap();
    assert_eq!(names.len(), 3);
    assert_eq!(names[0].as_str().unwrap(), "Alice");
    assert_eq!(names[1].as_str().unwrap(), "Bob");
    assert_eq!(names[2].as_str().unwrap(), "Charlie");

    let ages = processing::extract_json_values_by_key(json, "age").unwrap();
    assert_eq!(ages.len(), 3);
    assert_eq!(ages[0].as_u64().unwrap(), 30);
    assert_eq!(ages[1].as_u64().unwrap(), 25);
    assert_eq!(ages[2].as_u64().unwrap(), 35);

    // Test missing key
    let missing = processing::extract_json_values_by_key(json, "missing").unwrap();
    assert!(missing.is_empty());
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_extract_json_values_by_key_invalid() {
    let _ = processing::extract_json_values_by_key("invalid json", "key").unwrap();
}
