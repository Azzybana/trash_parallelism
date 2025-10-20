//! Tests for the serde module
use serde::{Deserialize, Serialize};
use trash_parallelism::serde::*;

#[test]
pub fn test_serialize_to_json() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let json = serialize_to_json(&data).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("42"));
}

#[test]
pub fn test_deserialize_from_json() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let json = r#"{"name":"test","value":42}"#;
    let data: TestStruct = deserialize_from_json(json).unwrap();
    assert_eq!(data.name, "test");
    assert_eq!(data.value, 42);
}

#[test]
pub fn test_pretty_json() {
    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let json = pretty_json(&data).unwrap();
    assert!(json.contains('\n'));
    assert!(json.contains("test"));
}

#[test]
pub fn test_validate_json() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let json = r#"{"name":"test","value":42}"#;
    let data: TestStruct = validate_json(json).unwrap();
    assert_eq!(data.name, "test");
    assert_eq!(data.value, 42);
}

#[test]
pub fn test_encode_base64() {
    let data = b"Hello, World!";
    let encoded = encode_base64(data);
    assert!(!encoded.is_empty());
    assert!(
        encoded
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    );
}

#[test]
pub fn test_decode_base64() {
    let encoded = "SGVsbG8sIFdvcmxkIQ==";
    let decoded = decode_base64(encoded).unwrap();
    assert_eq!(decoded, b"Hello, World!");
}

#[test]
pub fn test_serialize_to_base64_json() {
    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let b64_json = serialize_to_base64_json(&data).unwrap();
    assert!(!b64_json.is_empty());
}

#[test]
pub fn test_deserialize_from_base64_json() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let b64_json = serialize_to_base64_json(&data).unwrap();
    let decoded: TestStruct = deserialize_from_base64_json(&b64_json).unwrap();
    assert_eq!(decoded, data);
}

#[test]
pub fn test_serialize_to_bytes() {
    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let bytes = serialize_to_bytes(&data).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
pub fn test_deserialize_from_bytes() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let bytes = serialize_to_bytes(&data).unwrap();
    let decoded: TestStruct = deserialize_from_bytes(&bytes).unwrap();
    assert_eq!(decoded, data);
}

#[test]
pub fn test_hash_json_ahash() {
    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let hash = hash_json_ahash(&data).unwrap();
    assert!(hash > 0);
}

#[test]
pub fn test_serialize_with_timestamp() {
    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let json = serialize_with_timestamp(&data, "test_op").unwrap();
    assert!(json.contains("timestamp"));
    assert!(json.contains("test_op"));
    assert!(json.contains("data"));
}

#[test]
pub fn test_deserialize_with_timestamp() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    let json = serialize_with_timestamp(&data, "test_op").unwrap();
    let (decoded, _timestamp, operation): (TestStruct, _, String) =
        deserialize_with_timestamp(&json).unwrap();
    assert_eq!(decoded, data);
    assert_eq!(operation, "test_op");
}

#[test]
pub fn test_json_contains_key() {
    let json = r#"{"name":"test","value":42,"active":true}"#;
    assert!(json_contains_key(json, "name"));
    assert!(json_contains_key(json, "value"));
    assert!(json_contains_key(json, "active"));
    assert!(!json_contains_key(json, "missing"));
}

#[test]
pub fn test_extract_json_value() {
    let json = r#"{"name":"test","value":42,"active":true}"#;
    assert_eq!(
        extract_json_value(json, "name"),
        Some(r#""test""#.to_string())
    );
    assert_eq!(extract_json_value(json, "value"), Some("42".to_string()));
    assert_eq!(extract_json_value(json, "active"), Some("true".to_string()));
    assert_eq!(extract_json_value(json, "missing"), None);
}

#[test]
pub fn test_serialize_to_writer() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };

    let mut buffer = Vec::new();
    serialize_to_writer(&mut buffer, &data).unwrap();

    let json = String::from_utf8(buffer).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("42"));
}

#[test]
pub fn test_deserialize_from_reader() {
    use std::io::Cursor;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let json = r#"{"name":"test","value":42}"#;
    let reader = Cursor::new(json.as_bytes());
    let data: TestStruct = deserialize_from_reader(reader).unwrap();
    assert_eq!(data.name, "test");
    assert_eq!(data.value, 42);
}

#[test]
pub fn test_serialize_pretty_to_writer() {
    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };

    let mut buffer = Vec::new();
    serialize_pretty_to_writer(&mut buffer, &data).unwrap();

    let json = String::from_utf8(buffer).unwrap();
    assert!(json.contains('\n'));
    assert!(json.contains("test"));
    assert!(json.contains("42"));
}

#[test]
pub fn test_serialize_to_file_async() {
    use std::fs;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };

    let mut path = std::env::temp_dir();
    path.push("test_async.json");

    smol::block_on(async {
        serialize_to_file_async(&data, path.to_str().unwrap())
            .await
            .unwrap();
    });

    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("test"));
    assert!(contents.contains("42"));

    fs::remove_file(&path).unwrap();
}

#[test]
pub fn test_deserialize_from_file_async() {
    use std::fs;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };

    let mut path = std::env::temp_dir();
    path.push("test_async_read.json");
    let json = serde_json::to_string(&data).unwrap();
    fs::write(&path, json).unwrap();

    let loaded: TestStruct = smol::block_on(async {
        deserialize_from_file_async(path.to_str().unwrap())
            .await
            .unwrap()
    });

    assert_eq!(loaded, data);

    fs::remove_file(&path).unwrap();
}

#[test]
pub fn test_serialize_with_logging() {
    #[derive(Serialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    let data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };

    let json = serialize_with_logging(&data, "Test logging").unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("42"));
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_deserialize_from_json_invalid() {
    let _ = deserialize_from_json::<serde_json::Value>("invalid json").unwrap();
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_decode_base64_invalid() {
    let _ = decode_base64("invalid base64!").unwrap();
}

#[test]
pub fn test_serde() {
    test_serialize_to_json();
    test_deserialize_from_json();
    test_pretty_json();
    test_validate_json();
    test_encode_base64();
    test_decode_base64();
    test_serialize_to_base64_json();
    test_deserialize_from_base64_json();
    test_serialize_to_bytes();
    test_deserialize_from_bytes();
    test_hash_json_ahash();
    test_serialize_with_timestamp();
    test_deserialize_with_timestamp();
    test_json_contains_key();
    test_extract_json_value();
    test_serialize_to_writer();
    test_deserialize_from_reader();
    test_serialize_pretty_to_writer();
    test_serialize_to_file_async();
    test_deserialize_from_file_async();
    test_serialize_with_logging();
}
