use super::*;
use crate::api;
use serde_json::json;

#[test]
fn requires_page_metadata_and_preserves_exact_large_user_ids() {
    let rows =
        json!([{"id":"9007199254740993"},{"id":"9007199254740994"},{"id":"9007199254740995"}]);
    assert!(user_page(&rows).is_err());
    let page = user_page(&json!({"items":rows,"total":3,"page":1,"page_size":20})).unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(
        page.rows,
        json!([{"id":"9007199254740993"},{"id":"9007199254740994"},{"id":"9007199254740995"}])
            .as_array()
            .unwrap()
            .clone()
    );
}

#[test]
fn server_page_is_not_sliced_again() {
    let page =
        user_page(&json!({"items":[{"id":"42"}],"total":21,"page":2,"page_size":20})).unwrap();
    assert_eq!(page.total, 21);
    assert_eq!(page.rows.len(), 1);
    assert_eq!(api::text(&page.rows[0], "id"), "42");
}
