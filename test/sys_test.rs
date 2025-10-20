//! Tests for the sys module
use chrono::Datelike;
use trash_parallelism::sys::*;

#[test]
pub fn test_timer_creation() {
    let _timer = Timer::new("test_timer");
    // Timer prints on drop, expect no panic
}

#[test]
pub fn test_current_utc_time() {
    let now = current_utc_time();
    assert!(now.timestamp() > 0); // Should be a valid timestamp
}

#[test]
pub fn test_format_datetime() {
    let dt = current_utc_time();
    let formatted = format_datetime(&dt);
    assert!(formatted.contains('T')); // RFC3339 format has T
    assert!(formatted.ends_with('Z')); // UTC timezone
}

#[test]
pub fn test_parse_datetime() {
    let input = "2023-01-01T12:00:00Z";
    let dt = parse_datetime(input).unwrap();
    assert_eq!(dt.year(), 2023);
    assert_eq!(dt.month(), 1);
    assert_eq!(dt.day(), 1);
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_parse_datetime_invalid() {
    let _ = parse_datetime("invalid").unwrap();
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_parse_date_invalid() {
    let _ = parse_date("invalid").unwrap();
}

#[test]
pub fn test_parse_date() {
    let date = parse_date("2023-12-25").unwrap();
    assert_eq!(date.year(), 2023);
    assert_eq!(date.month(), 12);
    assert_eq!(date.day(), 25);
}

#[test]
pub fn test_serialize_deserialize_timestamp() {
    let dt = current_utc_time();
    let json = serialize_timestamp(&dt).unwrap();
    let restored = deserialize_timestamp(&json).unwrap();
    assert_eq!(dt, restored);
}

#[test]
fn test_deserialize_timestamp_invalid() {
    let result = deserialize_timestamp("invalid json");
    assert!(result.is_err());
}

#[test]
pub fn test_batch_parse_dates() {
    let dates = vec!["2023-01-01", "2023-01-02"];
    let parsed = batch_parse_dates(&dates).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].to_string(), "2023-01-01");
}

#[test]
pub fn test_convert_timezone_offset() {
    let dt = current_utc_time();
    let offset_dt = convert_timezone_offset(&dt, 5); // +5 hours
    let duration = offset_dt.signed_duration_since(dt);
    assert_eq!(duration.num_hours(), 5);
}

#[test]
pub fn test_read_env_var_or() {
    let value = read_env_var_or("NONEXISTENT_VAR", "default");
    assert_eq!(value, "default");
}

#[test]
pub fn test_read_env_var_parse() {
    unsafe {
        std::env::set_var("TEST_PORT", "8080");
    }
    let port: u16 = read_env_var_parse("TEST_PORT").unwrap();
    assert_eq!(port, 8080);
    unsafe {
        std::env::remove_var("TEST_PORT");
    }
}

#[test]
fn test_read_env_var_parse_invalid() {
    unsafe {
        std::env::set_var("TEST_INVALID", "not_a_number");
    }
    let result: Result<u16, Box<dyn std::error::Error>> = read_env_var_parse("TEST_INVALID");
    assert!(result.is_err());
    unsafe {
        std::env::remove_var("TEST_INVALID");
    }
}

#[test]
pub fn test_validate_required_env_vars() {
    let result = validate_required_env_vars(&["PATH"]); // PATH should exist
    assert!(result.is_ok());
}

#[test]
pub fn test_validate_required_env_vars_missing() {
    let result = validate_required_env_vars(&["NONEXISTENT_VAR_12345"]);
    assert!(result.is_err());
    let missing_vars = result.unwrap_err();
    assert_eq!(missing_vars, vec!["NONEXISTENT_VAR_12345".to_string()]);
}

#[test]
pub fn test_normalize_path() {
    let normalized = normalize_path("./foo/../bar");
    assert_eq!(normalized, "bar");
}

#[test]
pub fn test_join_paths() {
    let joined = join_paths("home", &["user", "docs"]);
    assert!(joined.contains("home"));
    assert!(joined.contains("user"));
    assert!(joined.contains("docs"));
}

#[test]
pub fn test_get_file_extension() {
    assert_eq!(get_file_extension("file.txt"), Some("txt".to_string()));
    assert_eq!(get_file_extension("file"), None);
}

#[test]
pub fn test_get_file_stem() {
    assert_eq!(get_file_stem("file.txt"), Some("file".to_string()));
    assert_eq!(get_file_stem("dir/"), None);
}

#[test]
pub fn test_is_file() {
    assert!(is_file("Cargo.toml"));
    assert!(!is_file("nonexistent"));
}

#[test]
pub fn test_is_directory() {
    assert!(is_directory("src"));
    assert!(!is_directory("Cargo.toml"));
}

#[test]
pub fn test_get_file_size() {
    let size = get_file_size("Cargo.toml").unwrap();
    assert!(size > 0);
}

#[test]
fn test_get_file_size_nonexistent() {
    let result = get_file_size("nonexistent_file_12345.txt");
    assert!(result.is_err());
}

#[test]
pub fn test_create_temp_file() {
    let temp = create_temp_file().unwrap();
    let path = temp.path().to_string_lossy().to_string();
    assert!(is_file(&path));
    // File is deleted on drop
}

#[test]
pub fn test_list_directory() {
    let entries = list_directory(".").unwrap();
    assert!(entries.contains(&"Cargo.toml".to_string()));
}

#[test]
fn test_list_directory_nonexistent() {
    let result = list_directory("nonexistent_directory_12345");
    assert!(result.is_err());
}

#[test]
pub fn test_timer_elapsed() {
    let timer = Timer::new("test_timer");
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = timer.elapsed();
    assert!(elapsed.as_millis() >= 10);
}

#[test]
pub fn test_read_env_var() {
    unsafe {
        std::env::set_var("TEST_VAR", "test_value");
    }
    let value = read_env_var("TEST_VAR").unwrap();
    assert_eq!(value, "test_value");
    unsafe {
        std::env::remove_var("TEST_VAR");
    }
}

#[test]
pub fn test_read_env_var_json() {
    unsafe {
        std::env::set_var("TEST_JSON", r#"{"key": "value"}"#);
    }
    let config: serde_json::Value = read_env_var_json("TEST_JSON").unwrap();
    assert_eq!(config["key"], "value");
    unsafe {
        std::env::remove_var("TEST_JSON");
    }
}

#[test]
fn test_read_env_var_json_invalid() {
    unsafe {
        std::env::set_var("TEST_INVALID_JSON", "not json");
    }
    let result: Result<serde_json::Value, Box<dyn std::error::Error>> =
        read_env_var_json("TEST_INVALID_JSON");
    assert!(result.is_err());
    unsafe {
        std::env::remove_var("TEST_INVALID_JSON");
    }
}

#[test]
pub fn test_read_env_vars_parallel() {
    unsafe {
        std::env::set_var("TEST_VAR1", "value1");
        std::env::set_var("TEST_VAR2", "value2");
    }
    let keys = vec!["TEST_VAR1", "TEST_VAR2", "NONEXISTENT"];
    let vars = read_env_vars_parallel(&keys);
    assert_eq!(vars.get("TEST_VAR1"), Some(&"value1".to_string()));
    assert_eq!(vars.get("TEST_VAR2"), Some(&"value2".to_string()));
    assert!(!vars.contains_key("NONEXISTENT"));
    unsafe {
        std::env::remove_var("TEST_VAR1");
        std::env::remove_var("TEST_VAR2");
    }
}

#[test]
pub fn test_get_file_metadata() {
    let metadata = get_file_metadata("Cargo.toml").unwrap();
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);
}

#[test]
fn test_get_file_metadata_nonexistent() {
    let result = get_file_metadata("nonexistent_file_12345.txt");
    assert!(result.is_err());
}

#[test]
pub fn test_get_file_modified_time() {
    let modified = get_file_modified_time("Cargo.toml").unwrap();
    // Just check it's a valid SystemTime, not in the future
    assert!(
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            > 0
    );
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_get_file_modified_time_nonexistent() {
    let _ = get_file_modified_time("nonexistent_file_12345.txt").unwrap();
}

#[test]
pub fn test_get_files_metadata_parallel() {
    let paths = vec!["Cargo.toml", "src/lib.rs"];
    let results = get_files_metadata_parallel(&paths);
    assert!(results.contains_key("Cargo.toml"));
    assert!(results["Cargo.toml"].is_ok());
    assert!(results.contains_key("src/lib.rs"));
    assert!(results["src/lib.rs"].is_ok());
}

#[test]
pub fn test_serialize_file_info() {
    let metadata = get_file_metadata("Cargo.toml").unwrap();
    let json = serialize_file_info(&metadata).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed["size"].is_number());
    assert_eq!(parsed["is_file"], true);
}

#[test]
pub fn test_deserialize_file_info() {
    let json = r#"{"size":1024,"is_file":true,"is_dir":false,"modified":"1234567890"}"#;
    let info = deserialize_file_info(json).unwrap();
    assert_eq!(info["size"], 1024);
    assert_eq!(info["is_file"], true);
}

#[test]
pub fn test_walk_directory() {
    let files = walk_directory("src").unwrap();
    assert!(!files.is_empty());
    assert!(files.iter().any(|f| f.ends_with("lib.rs")));
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_walk_directory_nonexistent() {
    let _ = walk_directory("nonexistent_directory_12345").unwrap();
}

#[test]
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub fn test_find_files_parallel() {
    let rs_files = find_files_parallel("src", "rs").unwrap();
    assert!(!rs_files.is_empty());
    assert!(rs_files.iter().all(|f| f.ends_with(".rs")));
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
fn test_find_files_parallel_nonexistent() {
    let _ = find_files_parallel("nonexistent_directory_12345", "rs").unwrap();
}

#[test]
pub fn test_sys() {
    test_timer_creation();
    test_current_utc_time();
    test_format_datetime();
    test_parse_datetime();
    test_parse_date();
    test_serialize_deserialize_timestamp();
    // negative tests that use should_panic are executed by the test runner
    // directly; do not call them from this aggregator to avoid panics here.
    test_batch_parse_dates();
    test_convert_timezone_offset();
    test_read_env_var_or();
    test_read_env_var_parse();
    test_validate_required_env_vars();
    test_validate_required_env_vars_missing();
    test_normalize_path();
    test_join_paths();
    test_get_file_extension();
    test_get_file_stem();
    test_is_file();
    test_is_directory();
    test_get_file_size();
    test_create_temp_file();
    test_list_directory();
    test_timer_elapsed();
    test_read_env_var();
    test_read_env_var_json();
    test_read_env_vars_parallel();
    test_get_file_metadata();
    test_get_file_modified_time();
    test_get_files_metadata_parallel();
    test_serialize_file_info();
    test_deserialize_file_info();
    test_walk_directory();
    test_find_files_parallel();
}
