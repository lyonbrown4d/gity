use uuid::Uuid;

/// Generate a sortable UUIDv7 for request, operation, or trace correlation.
pub fn new_operation_id() -> Uuid {
  Uuid::now_v7()
}
