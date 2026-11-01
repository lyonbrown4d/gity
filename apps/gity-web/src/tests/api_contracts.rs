use super::*;
#[test]
fn preserves_string_ids_without_float_coercion() {
    let item = json!({"id":"9223372036854775806"});
    assert_eq!(text(&item, "id"), "9223372036854775806");
}
#[test]
fn reads_paginated_and_bare_collections() {
    let rows = json!([{"id":"1"},{"id":"2"}]);
    assert_eq!(list(&rows).len(), 2);
    assert_eq!(list(&json!({"items":rows,"total":2})).len(), 2);
    assert!(list(&Value::Null).is_empty());
}
