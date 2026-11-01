//! Application-generated identifiers shared by all Gity components.

mod operation;
mod snowflake;

pub use operation::new_operation_id;
pub use snowflake::{configure_node, new_id};

#[cfg(test)]
mod tests;
