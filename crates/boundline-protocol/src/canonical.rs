//! Deterministic JSON serialization supports stable request digests without storing them.

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// Serializes a value with object keys sorted recursively and no whitespace.
///
/// The function deliberately performs no hashing or persistence so T011 can
/// own the idempotency store while consuming the same canonical byte sequence.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    let canonical = sort_object_keys(value);
    serde_json::to_string(&canonical)
}

fn sort_object_keys(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted = object
                .into_iter()
                .map(|(key, value)| (key, sort_object_keys(value)))
                .collect::<BTreeMap<_, _>>();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(sort_object_keys).collect()),
        scalar => scalar,
    }
}
