use super::create_volume_with;
use anyhow::{Result, bail};
use serde_json::json;
use std::cell::RefCell;

fn proof(invocation: &str) -> Vec<u8> {
  serde_json::to_vec(&json!([{"Name":"target_data","Labels":{
    "com.docker.compose.project":"target", "com.docker.compose.volume":"data",
    "io.gity.backup.restore":invocation
  }}]))
  .unwrap()
}

#[test]
fn failed_create_response_cleans_only_its_proven_allocation() {
  let calls = RefCell::new(Vec::new());
  let run = |args: &[&str], _: &str| -> Result<Vec<u8>> {
    calls
      .borrow_mut()
      .push(args.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    match args[1] {
      "create" => bail!("response lost after daemon allocation"),
      "inspect" => Ok(proof("invocation")),
      "rm" => Ok(Vec::new()),
      _ => unreachable!(),
    }
  };
  assert!(create_volume_with("target", "data", "invocation", &run, |_| Ok(false)).is_err());
  assert_eq!(
    calls.borrow().last().unwrap(),
    &vec!["volume", "rm", "target_data"]
  );
}

#[test]
fn malformed_create_identity_and_transient_inspection_failure_do_not_leak_owned_volume() {
  for malformed_identity in [true, false] {
    let inspections = RefCell::new(0);
    let removals = RefCell::new(0);
    let run = |args: &[&str], _: &str| -> Result<Vec<u8>> {
      match args[1] {
        "create" => Ok(if malformed_identity {
          b"unexpected".to_vec()
        } else {
          b"target_data".to_vec()
        }),
        "inspect" => {
          *inspections.borrow_mut() += 1;
          if !malformed_identity && *inspections.borrow() == 1 {
            bail!("transient inspect failure");
          }
          Ok(proof("invocation"))
        }
        "rm" => {
          *removals.borrow_mut() += 1;
          Ok(Vec::new())
        }
        _ => unreachable!(),
      }
    };
    assert!(create_volume_with("target", "data", "invocation", &run, |_| Ok(false)).is_err());
    assert_eq!(*removals.borrow(), 1);
  }
}

#[test]
fn concurrent_foreign_volume_and_unverifiable_volume_are_never_deleted() {
  for unverifiable in [false, true] {
    let removals = RefCell::new(0);
    let run = |args: &[&str], _: &str| -> Result<Vec<u8>> {
      match args[1] {
        "create" => Ok(b"target_data".to_vec()),
        "inspect" if unverifiable => bail!("Docker inspection unavailable"),
        "inspect" => Ok(proof("somebody-else")),
        "rm" => {
          *removals.borrow_mut() += 1;
          Ok(Vec::new())
        }
        _ => unreachable!(),
      }
    };
    assert!(create_volume_with("target", "data", "invocation", &run, |_| Ok(false)).is_err());
    assert_eq!(*removals.borrow(), 0);
  }
}

#[test]
fn successful_allocation_is_transferred_and_existing_volume_is_untouched() {
  let removals = RefCell::new(0);
  let run = |args: &[&str], _: &str| -> Result<Vec<u8>> {
    match args[1] {
      "create" => Ok(b"target_data".to_vec()),
      "inspect" => Ok(proof("invocation")),
      "rm" => {
        *removals.borrow_mut() += 1;
        Ok(Vec::new())
      }
      _ => unreachable!(),
    }
  };
  assert_eq!(
    create_volume_with("target", "data", "invocation", &run, |_| Ok(false)).unwrap(),
    "target_data"
  );
  assert_eq!(*removals.borrow(), 0);
  assert!(
    create_volume_with(
      "target",
      "data",
      "invocation",
      &|_, _| panic!("existing volume must not be accessed"),
      |_| Ok(true)
    )
    .is_err()
  );
}
