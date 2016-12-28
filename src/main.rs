#![feature(link_args)]

extern crate protobuf;

use std::ffi::CString;
use std::io::Read;
use std::os::raw::c_char;
use std::slice;

use protobuf::ProtobufResult;

use cursor::{Command, Cursor};
use storage::Storage;
use tag::{Value, get_tag_map};
use vector_tile::{Tile, Tile_GeomType};

pub mod cursor;
pub mod storage;
pub mod tag;
pub mod vector_tile;

pub fn process<R: Read>(mut r: R) -> ProtobufResult<String> {
    let tile: Tile = protobuf::parse_from_reader(&mut r)?;
    let mut storage = Storage::new();

    for layer in tile.get_layers() {
        let scale: f32 = 256f32 / layer.get_extent() as f32;
        let keys = layer.get_keys();
        let values = layer.get_values();
        for feature in layer.get_features() {
            if feature.get_field_type() == Tile_GeomType::LINESTRING {
                let tag_map = get_tag_map(keys, values, feature.get_tags())?;
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
                let cursor = Cursor::new(feature.get_geometry());
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

#[cfg_attr(target_arch="asmjs",
           link_args="-s EXPORTED_FUNCTIONS=['_process_web','_free_cstring_web']")]
extern {}

#[no_mangle]
pub extern fn process_web(p: *const u8, len: usize) -> *const c_char {
    let input = unsafe {
        slice::from_raw_parts(p, len)
    };
    let output = process(input).expect("input file is valid");
    CString::new(output).expect("string contains no internal NULL").into_raw()
}

#[no_mangle]
pub extern fn free_cstring_web(p: *mut c_char) {
    unsafe {
        if p.is_null() { return }
        CString::from_raw(p)
    };
}

fn main() {}
