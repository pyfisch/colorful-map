//! Describes tag values and provides the TagMap type.

use std::collections::HashMap;

use protobuf::{ProtobufError, ProtobufResult};

use vector_tile::Tile_Value;

/// A map to store tags of features.
pub type TagMap<'k, 'v> = HashMap<&'k str, Value<'v>>;

/// A tag value represents data associated to a key.
///
/// It may either be some kind of number, a string or a boolean.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Value<'a> {
    String(&'a str),
    Float32(f32),
    Float64(f64),
    Int64(i64),
    Uint64(u64),
    Bool(bool)
}

impl<'a> Value<'a> {
    /// Returns a Value for a Tile_Value.
    ///
    /// Panics if the value is empty.
    pub fn from_tile_value(value: &Tile_Value) -> ProtobufResult<Value> {
        use self::Value::*;
        Ok(if value.has_string_value() {
            String(value.get_string_value())
        } else if value.has_float_value() {
            Float32(value.get_float_value())
        } else if value.has_double_value() {
            Float64(value.get_double_value())
        } else if value.has_int_value() {
            Int64(value.get_int_value())
        } else if value.has_uint_value() {
            Uint64(value.get_uint_value())
        } else if value.has_sint_value() {
            Int64(value.get_sint_value())
        } else if value.has_bool_value() {
            Bool(value.get_bool_value())
        } else {
            return Err(ProtobufError::WireError(
                "mvt: A value must contain data.".to_owned()));
        })
    }

    /// Converts the value to u64, if possible.
    pub fn i64(&self) -> Option<i64> {
        use self::Value::*;
        match *self {
            Int64(x) => Some(x),
            Uint64(x) => Some(x as i64),
            _ => None,
        }
    }

    /// Converts the value to u16, if possible.
    pub fn u16(&self) -> Option<u16> {
        use self::Value::*;
        match *self {
            Int64(x) => Some(x as u16),
            Uint64(x) => Some(x as u16),
            _ => None,
        }
    }

    /// Converts the value to &str, if possible.
    pub fn str(&self) -> Option<&str> {
        use self::Value::*;
        match *self {
            String(x) => Some(x),
            _ => None,
        }
    }

    /// Returns true if the value "trueish".
    ///
    /// "Trueish" means either true or an integer
    /// different from zero.
    pub fn yes(&self) -> bool {
        use self::Value::*;
        match *self {
            Bool(true) => true,
            _ => self.i64().unwrap_or(0) != 0,
        }
    }

    /// Returns the value as f32, if possible.
    ///
    /// Note f64 is converted but no integer types.
    pub fn f32(&self) -> Option<f32> {
        use self::Value::*;
        match *self {
            Float32(x) => Some(x),
            Float64(x) => Some(x as f32),
            _ => None,
        }
    }
}
