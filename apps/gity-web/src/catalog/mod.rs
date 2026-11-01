//! Shared field definitions retained for the project creation form.
use crate::ui::{Field, field};

pub fn project_fields() -> Vec<Field> {
  vec![
    field("organization_id", "Organization", "org", true, ""),
    field("key", "Project key", "text", true, ""),
    field("name", "Name", "text", true, ""),
    field("description", "Description", "textarea", false, ""),
    field(
      "visibility",
      "Visibility",
      "select:private|internal|public",
      true,
      "private",
    ),
    field("default_branch", "Default branch", "text", true, "main"),
  ]
}
