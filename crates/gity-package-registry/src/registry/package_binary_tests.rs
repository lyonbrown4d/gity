use super::*;
#[test]
fn single_byte_range_bounds_are_safe() {
  assert_eq!(bounds("bytes=2-4", 10), Some((2, 5)));
  assert_eq!(bounds("bytes=-3", 10), Some((7, 10)));
  assert_eq!(bounds("bytes=8-", 10), Some((8, 10)));
  assert_eq!(bounds("bytes=10-", 10), None);
  assert_eq!(bounds("bytes=0-1,3-4", 10), None);
}
