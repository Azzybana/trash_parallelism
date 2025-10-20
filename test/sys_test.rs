//! Tests for the sys module

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use trash_utilities::sys::*;

    #[test]
    fn test_timer_creation() {
        let _timer = Timer::new("test_timer");
        // Timer prints on drop, expect no panic
    }

    #[test]
    fn test_current_utc_time() {
        let now = current_utc_time();
        assert!(now.timestamp() > 0); // Should be a valid timestamp
    }

    #[test]
    fn test_format_datetime() {
        let dt = current_utc_time();
        let formatted = format_datetime(&dt);
        assert!(formatted.contains('T')); // RFC3339 format has T
        assert!(formatted.ends_with('Z')); // UTC timezone
    }

    #[test]
    fn test_parse_datetime() {
        let input = "2023-01-01T12:00:00Z";
        let dt = parse_datetime(input).unwrap();
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2023-12-25").unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 25);
    }

    #[test]
    fn test_serialize_deserialize_timestamp() {
        let dt = current_utc_time();
        let json = serialize_timestamp(&dt).unwrap();
        let restored = deserialize_timestamp(&json).unwrap();
        assert_eq!(dt, restored);
    }

    #[test]
    fn test_batch_parse_dates() {
        let dates = vec!["2023-01-01", "2023-01-02"];
        let parsed = batch_parse_dates(&dates).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].to_string(), "2023-01-01");
    }

    #[test]
    fn test_convert_timezone_offset() {
        let dt = current_utc_time();
        let offset_dt = convert_timezone_offset(&dt, 5); // +5 hours
        let duration = offset_dt.signed_duration_since(dt);
        assert_eq!(duration.num_hours(), 5);
    }

    #[test]
    fn test_read_env_var_or() {
        let value = read_env_var_or("NONEXISTENT_VAR", "default");
        assert_eq!(value, "default");
    }

    #[test]
    fn test_read_env_var_parse() {
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
    fn test_validate_required_env_vars() {
        let result = validate_required_env_vars(&["HOME"]); // HOME should exist
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_path() {
        let normalized = normalize_path("./foo/../bar");
        assert_eq!(normalized, "bar");
    }

    #[test]
    fn test_join_paths() {
        let joined = join_paths("home", &["user", "docs"]);
        assert!(joined.contains("home"));
        assert!(joined.contains("user"));
        assert!(joined.contains("docs"));
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("file.txt"), Some("txt".to_string()));
        assert_eq!(get_file_extension("file"), None);
    }

    #[test]
    fn test_get_file_stem() {
        assert_eq!(get_file_stem("file.txt"), Some("file".to_string()));
        assert_eq!(get_file_stem("dir/"), None);
    }

    #[test]
    fn test_is_file() {
        assert!(is_file("Cargo.toml"));
        assert!(!is_file("nonexistent"));
    }

    #[test]
    fn test_is_directory() {
        assert!(is_directory("src"));
        assert!(!is_directory("Cargo.toml"));
    }

    #[test]
    fn test_get_file_size() {
        let size = get_file_size("Cargo.toml").unwrap();
        assert!(size > 0);
    }

    #[test]
    fn test_create_temp_file() {
        let temp = create_temp_file().unwrap();
        let path = temp.path().to_string_lossy().to_string();
        assert!(is_file(&path));
        // File is deleted on drop
    }

    #[test]
    fn test_list_directory() {
        let entries = list_directory(".").unwrap();
        assert!(entries.contains(&"Cargo.toml".to_string()));
    }
}
