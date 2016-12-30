use std::collections::HashMap;

use protobuf::{ProtobufError, ProtobufResult};

use vector_tile::Tile_Value;

pub type TagMap<'k, 'v> = HashMap<&'k str, Value<'v>>;

/// A tag value represents data associated to a key.
///
/// It may either be some kind of number, a string or a boolean.
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
        use Value::*;
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

    pub fn i64(&self) -> Option<i64> {
        use Value::*;
        match *self {
            Int64(x) => Some(x),
            Uint64(x) => Some(x as i64),
            _ => None,
        }
    }

    pub fn u16(&self) -> Option<u16> {
        use Value::*;
        match *self {
            Int64(x) => Some(x as u16),
            Uint64(x) => Some(x as u16),
            _ => None,
        }
    }

    pub fn str(&self) -> Option<&str> {
        use Value::*;
        match *self {
            String(x) => Some(x),
            _ => None,
        }
    }

    pub fn yes(&self) -> bool {
        use Value::*;
        match *self {
            Bool(true) => true,
            _ => self.i64().unwrap_or(0) != 0,
        }
    }

    pub fn f32(&self) -> Option<f32> {
        use Value::*;
        match *self {
            Float32(x) => Some(x),
            Float64(x) => Some(x as f32),
            _ => None,
        }
    }
}
