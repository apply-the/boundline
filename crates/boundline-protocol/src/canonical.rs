//! Versioned deterministic JSON serialization defines mutation request bytes.

mod value;

use serde::Serialize;
use serde::ser::Error as SerdeError;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use value::canonical_value;

/// Failure to produce an unambiguous canonical request representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalizationError {
    /// Mutation request canonicalization does not admit floating-point values.
    FloatingPointUnsupported,
    /// Two authored object entries emitted the same JSON key.
    DuplicateObjectKey(String),
    /// A non-floating value could not be represented as JSON.
    Serialization(String),
}

impl Display for CanonicalizationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FloatingPointUnsupported => {
                formatter.write_str("floating-point values are unsupported in canonical requests")
            }
            Self::DuplicateObjectKey(key) => {
                write!(formatter, "duplicate object key in canonical request: {key}")
            }
            Self::Serialization(message) => {
                write!(formatter, "canonical request serialization failed: {message}")
            }
        }
    }
}

impl std::error::Error for CanonicalizationError {}

impl SerdeError for CanonicalizationError {
    fn custom<T: Display>(message: T) -> Self {
        Self::Serialization(message.to_string())
    }
}

impl From<serde_json::Error> for CanonicalizationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error.to_string())
    }
}

/// Produces canonical JSON V1 for a serializable value.
///
/// Serde controls enum and optional-field representation. JSON null,
/// booleans, strings, and integers retain their JSON representation; maps are
/// recursively key-sorted and sequences retain their authored order.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, CanonicalizationError> {
    let canonical = sort_object_keys(canonical_value(value)?);
    Ok(serde_json::to_string(&canonical)?)
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
