extern crate protobuf;

use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Display};
use std::io::Read;
use std::ptr;

use protobuf::{parse_from_reader, ProtobufResult};

use cursor::{Command, Cursor};
use storage::{Storage, Rank};
use vector_tile::{Tile, Tile_GeomType, Tile_Value};

pub mod cursor;
pub mod storage;
pub mod vector_tile;


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
    fn from_tile_value(value: &Tile_Value) -> Value {
        use Value::*;
        if value.has_string_value() {
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
            panic!();
        }
    }
}

fn get_tag_map<'k, 'v>(keys: &'k[String], values: &'v[Tile_Value], tags: &[u32]) -> HashMap<&'k str, Value<'v>> {
    if tags.len() % 2 != 0 {
        panic!();
    }
    let mut map = HashMap::new();
    let mut tags_iter = tags.iter();
    for i in 0..(tags.len() / 2) {
        let k = *tags_iter.next().unwrap() as usize;
        let v = *tags_iter.next().unwrap() as usize;
        if k >= keys.len() || v >= values.len() {
            panic!();
        }
        map.insert(keys[k].as_str(), Value::from_tile_value(&values[v]));
    }
    map
}

pub fn process<R: Read>(mut r: R) -> ProtobufResult<String> {
    let tile: Tile = protobuf::parse_from_reader(&mut r)?;
    let mut storage = Storage::new();

    for layer in tile.get_layers() {
        let scale: f32 = 256f32 / layer.get_extent() as f32;
        let keys = layer.get_keys();
        let values = layer.get_values();
        for feature in layer.get_features() {
            if feature.get_field_type() == Tile_GeomType::LINESTRING {
                let tag_map = get_tag_map(keys, values, feature.get_tags());
                let rank_value = match tag_map["sort_rank"] {
                     Value::Uint64(x) => x as u16,
                     Value::Int64(x) => x as u16,
                     _ => 0,
                };
                let kind = match tag_map["kind"] {
                    Value::String(x) => x,
                    _ => ""
                };
                let mut rank = storage.select(rank_value);
                let mut cursor = Cursor::new(feature.get_geometry());
                rank.push_format(format_args!("<path class=\"{}\" d=\"", kind));
                for elem in cursor {
                    match elem {
                        Ok(Command::MoveTo(x, y)) => rank.push_format(
                            format_args!("m {} {} ", (x as f32 * scale), (y as f32 * scale))),
                        Ok(Command::LineTo(x, y)) => rank.push_format(
                            format_args!("l {} {} ", (x as f32 * scale), (y as f32 * scale))),
                        Ok(Command::ClosePath) => rank.push_str("Z "),
                        Err(e) => return Err(e),
                    }
                }
                rank.push_str("\"></path>\n");
            }
        }
    }
    Ok(String::from(storage))
}
