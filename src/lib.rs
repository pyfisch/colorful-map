extern crate protobuf;

use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Display};
use std::io::Read;
use std::ptr;

use protobuf::{parse_from_reader, ProtobufResult};
use vector_tile::{Tile, Tile_GeomType, Tile_Value};
use storage::{Storage, Rank};

pub mod vector_tile;
pub mod storage;

enum Command {
    MoveTo(i32, i32),
    LineTo(i32, i32),
    ClosePath,
}

struct Turtle<'a> {
    geometry: &'a [u32],
    pos: usize,
    id: u32,
    count: u32,
}

impl<'a> Turtle<'a> {
    fn new(geometry: &'a[u32]) -> Turtle<'a> {
        Turtle {
            geometry: geometry,
            pos: 0,
            id: 0,
            count: 0,
        }
    }
}

pub fn de_zigzag(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 1) as i32))
}

impl<'a> Iterator for Turtle<'a> {
    type Item = Command;

    fn next(&mut self) -> Option<Command> {
        use Command::*;
        if self.pos >= self.geometry.len() {
            return None;
        }
        if self.count == 0 {
            self.id = self.geometry[self.pos] & 0x7;
            self.count = self.geometry[self.pos] >> 3;
            self.pos += 1;
        }
        self.count -= 1;
        if self.id == 1 {
            if self.pos + 2 > self.geometry.len() {
                self.pos = ::std::usize::MAX;
                return None;
            }
            let x = de_zigzag(self.geometry[self.pos]);
            let y = de_zigzag(self.geometry[self.pos + 1]);
            self.pos += 2;
            Some(MoveTo(x, y))
        } else if self.id == 2 {
            if self.pos + 2 > self.geometry.len() {
                self.pos = ::std::usize::MAX;
                return None;
            }
            let x = de_zigzag(self.geometry[self.pos]);
            let y = de_zigzag(self.geometry[self.pos + 1]);
            self.pos += 2;
            Some(LineTo(x, y))
        } else if self.id == 7 {
            Some(ClosePath)
        } else {
            self.pos = ::std::usize::MAX;
            None
        }
    }
}

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
                // TODO: use real rank
                let mut rank = storage.select(rank_value);
                let mut turtle = Turtle::new(feature.get_geometry());
                rank.push_str("<path class=\"");
                rank.push_str(kind);
                rank.push_str("\" d=\"");
                for elem in turtle {
                    match elem {
                        Command::MoveTo(x, y) => {
                            rank.push_str("m ");
                            rank.push_str((x as f32 * scale).to_string().as_str());
                            rank.push(' ');
                            rank.push_str((y as f32 * scale).to_string().as_str());
                            rank.push(' ');
                        }
                        Command::LineTo(x, y) => {
                            rank.push_str("l ");
                            rank.push_str((x as f32 * scale).to_string().as_str());
                            rank.push(' ');
                            rank.push_str((y as f32 * scale).to_string().as_str());
                            rank.push(' ');
                        }
                        Command::ClosePath => {
                            rank.push_str("Z ");
                        }
                    }
                }
                rank.push_str("\"></path>\n");
            }
        }
    }
    Ok(String::from(storage))
}



#[test]
fn test_storage() {
    let mut storage = Storage::new();
    {
        let mut rank = storage.select(42);
        rank.push_str("middle rank, ");
    }
    {
        let mut rank = storage.select(17);
        rank.push_str("low ");
        rank.push_str("rank, ");
    }
    {
        let mut rank = storage.select(123);
        rank.push_str("upper rank");
    }
    assert_eq!(String::from(storage).as_str(), "low rank, middle rank, upper rank");
}
