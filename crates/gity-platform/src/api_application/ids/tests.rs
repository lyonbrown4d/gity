use serde_json::json;
#[test]
fn identifiers_in_body_query_nullable_refs_and_arrays_are_lossless_strings() {
  let mut schema = json!({"type":"object","properties":{"body":{"$ref":"#/$defs/Create"},"query":{"type":"object","properties":{"organization_id":{"type":["integer","null"]},"branch":{"type":"string"}}}},"$defs":{"Create":{"type":"object","properties":{"organization_id":{"type":"integer","format":"int64"},"user_ids":{"type":"array","items":{"type":"integer","format":"int64"}}}}}});
  super::public_schema(&mut schema);
  let mut value = json!({"body":{"organization_id":"9007199254740993","user_ids":["9007199254740995"]},"query":{"organization_id":null,"branch":"12345"}});
  let validator = jsonschema::validator_for(&schema).unwrap();
  assert!(validator.is_valid(&value));
  super::native_arguments(&schema, &mut value).unwrap();
  assert_eq!(value["body"]["organization_id"], 9007199254740993_i64);
  assert_eq!(value["body"]["user_ids"][0], 9007199254740995_i64);
  assert_eq!(value["query"]["branch"], "12345");
  assert!(value["query"]["organization_id"].is_null());
}
