mod object;
mod refs;
mod repository;
mod transport;

pub use repository::GitRepository; // re-export for external use

pub mod http; // helpers for serving Git over HTTP

pub fn add(left: u64, right: u64) -> u64 {
  left + right
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
