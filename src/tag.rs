use std::collections::HashMap;

use protobuf::{ProtobufError, ProtobufResult};

use vector_tile::Tile_Value;

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
    fn from_tile_value(value: &Tile_Value) -> ProtobufResult<Value> {
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
}

pub fn get_tag_map<'k, 'v>(keys: &'k[String], values: &'v[Tile_Value], tags: &[u32])
        -> ProtobufResult<HashMap<&'k str, Value<'v>>> {
    if tags.len() % 2 != 0 {
        return Err(ProtobufError::WireError(
            "mvt: A tag list must be an even number of integers.".to_owned()));
    }
    let mut map = HashMap::new();
    for i in 0..(tags.len() / 2) {
        let k = tags[i * 2] as usize;
        let v = tags[i * 2 + 1] as usize;
        if k >= keys.len() || v >= values.len() {
            return Err(ProtobufError::WireError(
                "mvt: There is no such tag key/value.".to_owned()));
        }
        map.insert(keys[k].as_str(), Value::from_tile_value(&values[v])?);
    }
    Ok(map)
}
