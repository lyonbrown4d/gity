use std::sync::{Arc, OnceLock};
use std::time::Duration;

use ferroid::generator::{LockSnowflakeGenerator, Poll};
use ferroid::id::{SnowflakeTwitterId, ToU64};
use ferroid::time::{MonotonicClock, TimeSource};

/// Milliseconds since 2024-01-01T00:00:00Z fill the 41-bit timestamp field.
const EPOCH: Duration = Duration::from_millis(1_704_067_200_000);
/// Writers sharing one database must configure distinct node ids.
pub const MAX_NODE_ID: u16 = 1023;
/// Absorb sequence exhaustion and small clock steps instead of spinning forever.
const MAX_YIELD_ATTEMPTS: u32 = 64;
/// Clock steps beyond this are reported instead of silently absorbed.
const MAX_CLOCK_STEP_MILLIS: u64 = 1_000;

static INSTALLED: OnceLock<IdGenerator> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdError {
  /// `database.node_id` is outside `0..=1023`.
  InvalidNodeId(u16),
  /// Another node id already installed this process's generator.
  NodeConflict { installed: u16, requested: u16 },
  /// The sequence is exhausted and the clock has not advanced.
  Exhausted,
  /// The clock moved backwards by this many milliseconds.
  ClockBehind(u64),
  /// The generated value does not fit a signed 64-bit primary key.
  OutOfRange,
  /// The generator's lock was poisoned by a panicking writer.
  LockPoisoned,
}

impl std::fmt::Display for IdError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidNodeId(node) => {
        write!(
          formatter,
          "database.node_id {node} must be in 0..={MAX_NODE_ID}"
        )
      }
      Self::NodeConflict {
        installed,
        requested,
      } => write!(
        formatter,
        "Snowflake node {installed} is already installed in this process, cannot install {requested}"
      ),
      Self::Exhausted => {
        formatter.write_str("Snowflake sequence exhausted for the current millisecond")
      }
      Self::ClockBehind(millis) => {
        write!(formatter, "system clock moved backwards by {millis} ms")
      }
      Self::OutOfRange => {
        formatter.write_str("Snowflake identifier does not fit a signed 64-bit id")
      }
      Self::LockPoisoned => formatter.write_str("Snowflake generator lock poisoned"),
    }
  }
}

impl std::error::Error for IdError {}

/// Snowflake generator: 41-bit millisecond timestamp, 10-bit node, 12-bit
/// sequence, always a positive signed 64-bit value.
///
/// Clones share one sequence, so installing a generator and handing a clone to
/// the container cannot produce duplicate identifiers.
pub struct IdGenerator<T: TimeSource<u64> = MonotonicClock<1>> {
  node: u16,
  generator: Arc<LockSnowflakeGenerator<SnowflakeTwitterId, T>>,
}

impl<T: TimeSource<u64>> Clone for IdGenerator<T> {
  fn clone(&self) -> Self {
    Self {
      node: self.node,
      generator: Arc::clone(&self.generator),
    }
  }
}

impl<T: TimeSource<u64>> std::fmt::Debug for IdGenerator<T> {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("IdGenerator")
      .field("node", &self.node)
      .finish_non_exhaustive()
  }
}

impl<T: TimeSource<u64>> IdGenerator<T> {
  /// Builds a generator for `node` over an injected time source.
  pub fn with_clock(node: u16, clock: T) -> Result<Self, IdError> {
    if node > MAX_NODE_ID {
      return Err(IdError::InvalidNodeId(node));
    }
    Ok(Self {
      node,
      generator: Arc::new(LockSnowflakeGenerator::new(u64::from(node), clock)),
    })
  }
  pub fn node(&self) -> u16 {
    self.node
  }
  /// Generates the next identifier, waiting briefly for the clock to advance.
  ///
  /// Sequence exhaustion inside one millisecond and small backward clock steps
  /// are absorbed by short sleeps; larger steps and stalls are reported.
  pub fn next(&self) -> Result<i64, IdError> {
    for _ in 0..MAX_YIELD_ATTEMPTS {
      let poll = self
        .generator
        .try_poll_id()
        .map_err(|_| IdError::LockPoisoned)?;
      match poll {
        Poll::Ready { id } => {
          return i64::try_from(id.to_raw()).map_err(|_| IdError::OutOfRange);
        }
        Poll::Pending { yield_for } => {
          let wait = yield_for.to_u64();
          if wait > MAX_CLOCK_STEP_MILLIS {
            return Err(IdError::ClockBehind(wait));
          }
          std::thread::sleep(Duration::from_millis(wait.max(1)));
        }
      }
    }
    Err(IdError::Exhausted)
  }
}

impl IdGenerator<MonotonicClock<1>> {
  /// Builds the production generator aligned to Gity's 2024 epoch.
  pub fn new(node: u16) -> Result<Self, IdError> {
    Self::with_clock(node, MonotonicClock::<1>::with_epoch(EPOCH))
  }
}

/// Installs this process's generator. Repeated installs must agree on the node.
pub fn install(generator: IdGenerator) -> Result<(), IdError> {
  install_in(&INSTALLED, generator)
}

fn install_in(slot: &OnceLock<IdGenerator>, generator: IdGenerator) -> Result<(), IdError> {
  let requested = generator.node();
  let installed = slot.get_or_init(|| generator);
  if installed.node() == requested {
    Ok(())
  } else {
    Err(IdError::NodeConflict {
      installed: installed.node(),
      requested,
    })
  }
}

/// Generates an entity primary key, panicking only on an unrecoverable clock fault.
pub fn new_id() -> i64 {
  try_new_id().unwrap_or_else(|error| panic!("Snowflake identifier generation failed: {error}"))
}

/// Fallible variant for persistence hooks that must surface the failure.
pub fn try_new_id() -> Result<i64, IdError> {
  match INSTALLED.get() {
    Some(generator) => generator.next(),
    None => {
      tracing::warn!(
        node = 1,
        "Snowflake generator was not installed during assembly; defaulting to node 1"
      );
      let generator = INSTALLED
        .get_or_init(|| IdGenerator::new(1).expect("default Snowflake node 1 is always valid"));
      generator.next()
    }
  }
}

#[cfg(test)]
mod tests;
