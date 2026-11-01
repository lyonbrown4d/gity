use super::*;

#[test]
fn user_labels_preserve_large_ids_and_original_display_name_convention() {
  let users = json!([{"id":"9007199254740993","username":"alice","display_name":" Alice "}]);
  assert_eq!(user_label(&users, "9007199254740993"), "Alice (@alice)");
  assert_eq!(user_label(&users, "9007199254740994"), "#9007199254740994");
  assert_eq!(user_label(&users, ""), "Unknown user");
}

#[test]
fn label_changes_keep_existing_colors_and_deduplicate_trimmed_names() {
  let labels = vec![
    json!({"name":" bug ","color":" #f00 "}),
    json!({"name":"task","color":"#fff"}),
  ];
  assert_eq!(
    labels_body(labels.clone(), None, Some(("bug", "#0f0"))),
    json!({"labels":[{"name":"bug","color":"#f00"},{"name":"task","color":"#fff"}]})
  );
  assert_eq!(
    labels_body(labels, Some("task"), Some((" ", "#000"))),
    json!({"labels":[{"name":"bug","color":"#f00"}]})
  );
}
