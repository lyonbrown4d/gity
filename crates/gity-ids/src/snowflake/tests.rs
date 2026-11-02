use super::*;
use std::collections::HashSet;
use std::sync::Barrier;

#[test]
fn simultaneous_installations_share_one_sequence_and_reject_a_different_node() {
  let slot = OnceLock::new();
  let barrier = Barrier::new(16);
  let ids = std::thread::scope(|scope| {
    let handles = (0..16)
      .map(|_| {
        scope.spawn(|| {
          let generator = IdGenerator::new(7).unwrap();
          barrier.wait();
          install_in(&slot, generator).unwrap();
          (0..64)
            .map(|_| slot.get().unwrap().next().unwrap())
            .collect::<Vec<_>>()
        })
      })
      .collect::<Vec<_>>();
    handles
      .into_iter()
      .flat_map(|handle| handle.join().unwrap())
      .collect::<HashSet<_>>()
  });
  assert_eq!(ids.len(), 16 * 64);
  assert_eq!(
    install_in(&slot, IdGenerator::new(8).unwrap()),
    Err(IdError::NodeConflict {
      installed: 7,
      requested: 8,
    })
  );
}
