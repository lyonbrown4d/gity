use super::*;
use serde_json::json;

#[test]
fn page_contract_rejects_unpaginated_and_wrong_page_responses() {
    assert!(matches!(parse_page(json!([]), 1), Err(PageError::Invalid)));
    assert!(matches!(
        parse_page(json!({"items":[],"total":0,"page":2,"page_size":100}), 1),
        Err(PageError::Invalid)
    ));
    assert!(matches!(
        parse_page(json!({"items":[],"total":0,"page":1,"page_size":20}), 1),
        Err(PageError::Invalid)
    ));
}

#[test]
fn pages_append_every_user_without_coercing_large_identifiers() {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    let first = (0..100u64)
        .map(|offset| json!({"id":(9_007_199_254_740_993 + offset).to_string()}))
        .collect::<Vec<_>>();
    let parsed = parse_page(
        json!({"items":first,"total":101,"page":1,"page_size":100}),
        1,
    )
    .unwrap();
    append_page(&mut rows, &mut seen, parsed, 101).unwrap();
    let second = parse_page(
        json!({"items":[{"id":"9007199254741093"}],"total":101,"page":2,"page_size":100}),
        2,
    )
    .unwrap();
    append_page(&mut rows, &mut seen, second, 101).unwrap();
    assert_eq!(rows.len(), 101);
    assert_eq!(text(&rows[100], "id"), "9007199254741093");
}

#[test]
fn concurrent_directory_changes_never_return_a_partial_success() {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    let first = parse_page(
        json!({"items":[{"id":"1"}],"total":2,"page":1,"page_size":100}),
        1,
    )
    .unwrap();
    append_page(&mut rows, &mut seen, first, 2).unwrap();
    let duplicate = parse_page(
        json!({"items":[{"id":"1"}],"total":2,"page":2,"page_size":100}),
        2,
    )
    .unwrap();
    assert_eq!(
        append_page(&mut rows, &mut seen, duplicate, 2),
        Err(PageError::Changed)
    );
    let changed = parse_page(
        json!({"items":[{"id":"2"}],"total":3,"page":2,"page_size":100}),
        2,
    )
    .unwrap();
    assert_eq!(
        append_page(&mut rows, &mut seen, changed, 2),
        Err(PageError::Changed)
    );
}
