use serde_json::json;
use umami_cli::output::val_str;

#[test]
fn val_str_extracts_string() {
    let v = json!({ "name": "hello" });
    assert_eq!(val_str(&v, "name"), "hello");
}

#[test]
fn val_str_extracts_number() {
    let v = json!({ "count": 42 });
    assert_eq!(val_str(&v, "count"), "42");
}

#[test]
fn val_str_extracts_bool() {
    let v = json!({ "active": true });
    assert_eq!(val_str(&v, "active"), "true");
}

#[test]
fn val_str_missing_key_returns_dash() {
    let v = json!({ "name": "hello" });
    assert_eq!(val_str(&v, "missing"), "—");
}

#[test]
fn val_str_null_returns_dash() {
    let v = json!({ "name": null });
    assert_eq!(val_str(&v, "name"), "—");
}

#[test]
fn val_str_nested_object_returns_json() {
    let v = json!({ "nested": { "a": 1 } });
    let result = val_str(&v, "nested");
    assert!(result.contains("\"a\""));
}
