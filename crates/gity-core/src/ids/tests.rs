#[test]
fn ids_are_ordered_and_unique() {
  let values: Vec<_> = (0..10000).map(|_| super::new_id()).collect();
  assert!(values.windows(2).all(|p| p[0] < p[1]));
}
