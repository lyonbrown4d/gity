use super::*;

#[test]
fn local_references_link_to_the_current_project_without_losing_surrounding_text() {
  assert_eq!(
    link_references("Fix #0012 and (#7).", "/app/projects/1/2"),
    "Fix [#12](/app/projects/1/2/issues/12) and ([#7](/app/projects/1/2/issues/7))."
  );
}

#[test]
fn zero_embedded_and_overflow_references_remain_literal() {
  let text = "#0 user#12 #999999999999999999999999999999";
  assert_eq!(link_references(text, "/app/projects/1/2"), text);
}
