//! Application-generated identifiers shared by all Gity components.

mod operation;
mod snowflake;

pub use operation::new_operation_id;
pub use snowflake::{IdError, IdGenerator, MAX_NODE_ID, install, new_id, try_new_id};

#[cfg(test)]
mod tests;
