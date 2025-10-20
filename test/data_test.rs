//! Tests for the data module
use trash_utilities::data::*;

#[test]
pub fn test_parse_key_value_basic() {
    let content = "key1=value1\nkey2=value2\nkey3=value3";
    let map = parse_key_value(content);

    assert_eq!(map.get("key1"), Some(&"value1".to_string()));
    assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    assert_eq!(map.get("key3"), Some(&"value3".to_string()));
    assert_eq!(map.len(), 3);
}

#[test]
pub fn test_parse_key_value_with_whitespace() {
    let content = "  key1  =  value1  \n  key2=value2\nkey3=  value3  ";
    let map = parse_key_value(content);

    assert_eq!(map.get("key1"), Some(&"value1".to_string()));
    assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    assert_eq!(map.get("key3"), Some(&"value3".to_string()));
    assert_eq!(map.len(), 3);
}

#[test]
pub fn test_parse_key_value_empty_lines() {
    let content = "key1=value1\n\nkey2=value2\n\n\nkey3=value3\n";
    let map = parse_key_value(content);

    assert_eq!(map.get("key1"), Some(&"value1".to_string()));
    assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    assert_eq!(map.get("key3"), Some(&"value3".to_string()));
    assert_eq!(map.len(), 3);
}

#[test]
pub fn test_parse_key_value_no_equals() {
    let content = "key1=value1\ninvalid line\nkey2=value2";
    let map = parse_key_value(content);

    assert_eq!(map.get("key1"), Some(&"value1".to_string()));
    assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    assert_eq!(map.len(), 2);
}

#[test]
pub fn test_parse_key_value_multiple_equals() {
    let content = "key1=value=with=equals\nkey2=value2";
    let map = parse_key_value(content);

    assert_eq!(map.get("key1"), Some(&"value=with=equals".to_string()));
    assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    assert_eq!(map.len(), 2);
}

#[test]
pub fn test_parse_key_value_empty_values() {
    let content = "key1=\nkey2=value2\nkey3=";
    let map = parse_key_value(content);

    assert_eq!(map.get("key1"), Some(&"".to_string()));
    assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    assert_eq!(map.get("key3"), Some(&"".to_string()));
    assert_eq!(map.len(), 3);
}

#[test]
pub fn test_parse_key_value_config_format() {
    let content = r"
app.name=MyApp
app.version=1.0.0
database.host=localhost
database.port=5432
features.logging=true
features.metrics=false
";
    let map = parse_key_value(content);

    assert_eq!(map.get("app.name"), Some(&"MyApp".to_string()));
    assert_eq!(map.get("app.version"), Some(&"1.0.0".to_string()));
    assert_eq!(map.get("database.host"), Some(&"localhost".to_string()));
    assert_eq!(map.get("database.port"), Some(&"5432".to_string()));
    assert_eq!(map.get("features.logging"), Some(&"true".to_string()));
    assert_eq!(map.get("features.metrics"), Some(&"false".to_string()));
    assert_eq!(map.len(), 6);
}

#[test]
pub fn test_parse_key_value_env_format() {
    let content = "
HOME=/home/user
PATH=/usr/bin:/bin
USER=testuser
SHELL=/bin/bash
";
    let map = parse_key_value(content);

    assert_eq!(map.get("HOME"), Some(&"/home/user".to_string()));
    assert_eq!(map.get("PATH"), Some(&"/usr/bin:/bin".to_string()));
    assert_eq!(map.get("USER"), Some(&"testuser".to_string()));
    assert_eq!(map.get("SHELL"), Some(&"/bin/bash".to_string()));
    assert_eq!(map.len(), 4);
}

#[test]
pub fn test_parse_key_value_docker_format() {
    let content = "
POSTGRES_DB=myapp
POSTGRES_USER=admin
POSTGRES_PASSWORD=secret123
DATABASE_URL=postgresql://admin:secret123@localhost:5432/myapp
";
    let map = parse_key_value(content);

    assert_eq!(map.get("POSTGRES_DB"), Some(&"myapp".to_string()));
    assert_eq!(map.get("POSTGRES_USER"), Some(&"admin".to_string()));
    assert_eq!(map.get("POSTGRES_PASSWORD"), Some(&"secret123".to_string()));
    assert_eq!(
        map.get("DATABASE_URL"),
        Some(&"postgresql://admin:secret123@localhost:5432/myapp".to_string())
    );
    assert_eq!(map.len(), 4);
}

#[test]
pub fn test_parse_key_value_empty_string() {
    let content = "";
    let map = parse_key_value(content);
    assert!(map.is_empty());
}

#[test]
pub fn test_parse_key_value_only_newlines() {
    let content = "\n\n\n";
    let map = parse_key_value(content);
    assert!(map.is_empty());
}

#[test]
pub fn test_parse_key_value_single_line() {
    let content = "single=value";
    let map = parse_key_value(content);

    assert_eq!(map.get("single"), Some(&"value".to_string()));
    assert_eq!(map.len(), 1);
}

#[test]
pub fn test_parse_key_value_windows_line_endings() {
    let content = "key1=value1\r\nkey2=value2\r\nkey3=value3";
    let map = parse_key_value(content);

    assert_eq!(map.get("key1"), Some(&"value1".to_string()));
    assert_eq!(map.get("key2"), Some(&"value2".to_string()));
    assert_eq!(map.get("key3"), Some(&"value3".to_string()));
    assert_eq!(map.len(), 3);
}

#[test]
pub fn test_parse_key_value_duplicate_keys() {
    let content = "key=first\nkey=second\nkey=third";
    let map = parse_key_value(content);

    // Last value should win
    assert_eq!(map.get("key"), Some(&"third".to_string()));
    assert_eq!(map.len(), 1);
}

#[test]
pub fn test_parse_key_value_special_characters() {
    let content = "key1=value with spaces\nkey2=value-with-dashes\nkey3=value_with_underscores\nkey4=value.with.dots";
    let map = parse_key_value(content);

    assert_eq!(map.get("key1"), Some(&"value with spaces".to_string()));
    assert_eq!(map.get("key2"), Some(&"value-with-dashes".to_string()));
    assert_eq!(map.get("key3"), Some(&"value_with_underscores".to_string()));
    assert_eq!(map.get("key4"), Some(&"value.with.dots".to_string()));
    assert_eq!(map.len(), 4);
}

#[test]
pub fn test_data() {
    test_parse_key_value_basic();
    test_parse_key_value_with_whitespace();
    test_parse_key_value_empty_lines();
    test_parse_key_value_no_equals();
    test_parse_key_value_multiple_equals();
    test_parse_key_value_empty_values();
    test_parse_key_value_config_format();
    test_parse_key_value_env_format();
    test_parse_key_value_docker_format();
    test_parse_key_value_empty_string();
    test_parse_key_value_only_newlines();
    test_parse_key_value_single_line();
    test_parse_key_value_windows_line_endings();
    test_parse_key_value_duplicate_keys();
    test_parse_key_value_special_characters();
}
