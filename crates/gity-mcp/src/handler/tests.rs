use serde_json::{Value, json};
#[test]
fn wrapping_structured_results_preserves_document_root_references() {
  let original = json!({"type":"array","items":{"$ref":"#/$defs/Item"},"$defs":{"Item":{"type":"object","properties":{"id":{"type":"string"}},"required":["id"]}}});
  let schema = Value::Object(super::wrapped_output_schema(original));
  let validator = jsonschema::validator_for(&schema).unwrap();
  assert!(validator.is_valid(&json!({"result":[{"id":"9007199254740993"}]})));
  assert!(!validator.is_valid(&json!({"result":[{"id":42}]})));
}
