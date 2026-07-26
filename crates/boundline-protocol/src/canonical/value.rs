//! A single Serde traversal preserves JSON semantics while rejecting ambiguity.

use serde::Serialize;
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use serde_json::{Map, Number, Value};
use std::fmt::Display;

use super::CanonicalizationError;

/// Serializes exactly once so stateful serializers cannot change between validation and encoding.
pub(super) fn canonical_value<T: Serialize>(value: &T) -> Result<Value, CanonicalizationError> {
    value.serialize(ValueSerializer)
}

struct ValueSerializer;

enum ValueCompound {
    Sequence(Vec<Value>),
    Object { entries: Map<String, Value>, pending_key: Option<String> },
    TupleVariant { variant: String, values: Vec<Value> },
    StructVariant { variant: String, fields: Map<String, Value> },
}

impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = CanonicalizationError;
    type SerializeSeq = ValueCompound;
    type SerializeTuple = ValueCompound;
    type SerializeTupleStruct = ValueCompound;
    type SerializeTupleVariant = ValueCompound;
    type SerializeMap = ValueCompound;
    type SerializeStruct = ValueCompound;
    type SerializeStructVariant = ValueCompound;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(value))
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::from(value)))
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        Number::from_i128(value).map(Value::Number).ok_or_else(|| {
            CanonicalizationError::Serialization("integer is outside the JSON range".to_owned())
        })
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(value))
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(Number::from(value)))
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        Number::from_u128(value).map(Value::Number).ok_or_else(|| {
            CanonicalizationError::Serialization("integer is outside the JSON range".to_owned())
        })
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(CanonicalizationError::FloatingPointUnsupported)
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(CanonicalizationError::FloatingPointUnsupported)
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Array(value.iter().map(|byte| Value::Number(Number::from(*byte))).collect()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let mut object = Map::new();
        object.insert(variant.to_owned(), value.serialize(self)?);
        Ok(Value::Object(object))
    }

    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ValueCompound::Sequence(Vec::with_capacity(length.unwrap_or(0))))
    }

    fn serialize_tuple(self, length: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(length))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(length))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(ValueCompound::TupleVariant {
            variant: variant.to_owned(),
            values: Vec::with_capacity(length),
        })
    }

    fn serialize_map(self, length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ValueCompound::Object {
            entries: Map::with_capacity(length.unwrap_or(0)),
            pending_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(length))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(ValueCompound::StructVariant {
            variant: variant.to_owned(),
            fields: Map::with_capacity(length),
        })
    }

    fn collect_str<T: ?Sized + Display>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(value.to_string()))
    }
}

impl ValueCompound {
    fn push(&mut self, value: Value) -> Result<(), CanonicalizationError> {
        match self {
            Self::Sequence(values) | Self::TupleVariant { values, .. } => {
                values.push(value);
                Ok(())
            }
            Self::Object { .. } | Self::StructVariant { .. } => Err(
                CanonicalizationError::Serialization("sequence value used in object".to_owned()),
            ),
        }
    }

    fn insert(&mut self, key: String, value: Value) -> Result<(), CanonicalizationError> {
        let fields = match self {
            Self::Object { entries, .. } => entries,
            Self::StructVariant { fields, .. } => fields,
            Self::Sequence(_) | Self::TupleVariant { .. } => {
                return Err(CanonicalizationError::Serialization(
                    "object value used in sequence".to_owned(),
                ));
            }
        };
        if fields.contains_key(&key) {
            return Err(CanonicalizationError::DuplicateObjectKey(key));
        }
        fields.insert(key, value);
        Ok(())
    }

    fn finish(self) -> Result<Value, CanonicalizationError> {
        match self {
            Self::Sequence(values) => Ok(Value::Array(values)),
            Self::Object { entries, pending_key } => {
                if pending_key.is_some() {
                    return Err(CanonicalizationError::Serialization(
                        "map key has no value".to_owned(),
                    ));
                }
                Ok(Value::Object(entries))
            }
            Self::TupleVariant { variant, values } => {
                Ok(single_entry_object(variant, Value::Array(values)))
            }
            Self::StructVariant { variant, fields } => {
                Ok(single_entry_object(variant, Value::Object(fields)))
            }
        }
    }
}

impl SerializeSeq for ValueCompound {
    type Ok = Value;
    type Error = CanonicalizationError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.push(value.serialize(ValueSerializer)?)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

macro_rules! sequence_compound {
    ($trait:ident, $method:ident) => {
        impl $trait for ValueCompound {
            type Ok = Value;
            type Error = CanonicalizationError;

            fn $method<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
                self.push(value.serialize(ValueSerializer)?)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.finish()
            }
        }
    };
}

sequence_compound!(SerializeTuple, serialize_element);
sequence_compound!(SerializeTupleStruct, serialize_field);
sequence_compound!(SerializeTupleVariant, serialize_field);

impl SerializeMap for ValueCompound {
    type Ok = Value;
    type Error = CanonicalizationError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let serialized_key = map_key(key.serialize(ValueSerializer)?)?;
        match self {
            Self::Object { entries, pending_key } => {
                if pending_key.is_some() {
                    return Err(CanonicalizationError::Serialization(
                        "map key has no value".to_owned(),
                    ));
                }
                if entries.contains_key(&serialized_key) {
                    return Err(CanonicalizationError::DuplicateObjectKey(serialized_key));
                }
                *pending_key = Some(serialized_key);
                Ok(())
            }
            Self::Sequence(_) | Self::TupleVariant { .. } | Self::StructVariant { .. } => Err(
                CanonicalizationError::Serialization("map key used outside an object".to_owned()),
            ),
        }
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = match self {
            Self::Object { pending_key, .. } => pending_key.take().ok_or_else(|| {
                CanonicalizationError::Serialization("map value has no key".to_owned())
            })?,
            Self::Sequence(_) | Self::TupleVariant { .. } | Self::StructVariant { .. } => {
                return Err(CanonicalizationError::Serialization(
                    "map value used outside an object".to_owned(),
                ));
            }
        };
        self.insert(key, value.serialize(ValueSerializer)?)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl SerializeStruct for ValueCompound {
    type Ok = Value;
    type Error = CanonicalizationError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.insert(key.to_owned(), value.serialize(ValueSerializer)?)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl SerializeStructVariant for ValueCompound {
    type Ok = Value;
    type Error = CanonicalizationError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.insert(key.to_owned(), value.serialize(ValueSerializer)?)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

fn map_key(value: Value) -> Result<String, CanonicalizationError> {
    match value {
        Value::String(value) => Ok(value),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null | Value::Array(_) | Value::Object(_) => {
            Err(CanonicalizationError::Serialization("map key is not a JSON object key".to_owned()))
        }
    }
}

fn single_entry_object(key: String, value: Value) -> Value {
    let mut object = Map::new();
    object.insert(key, value);
    Value::Object(object)
}
