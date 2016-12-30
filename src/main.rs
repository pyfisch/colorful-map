//! Parse Mapbox Vector Tile (.mvt) files and return their
//! contents as a Scalable Vector Graphic (.svg) fragment.
//!
//! Provides a single function `process(r: Read)` to convert
//! files.
//!
//! Intended to be run in the browser.

#![deny(missing_docs)]
// Exported functions are passed to the asm.js linker.
#![feature(link_args)]

extern crate protobuf;

use std::ffi::CString;
use std::io::Read;
use std::os::raw::c_char;
use std::slice;

use protobuf::ProtobufResult;

use storage::Storage;
use layer::Layer;
use vector_tile::Tile;

pub mod cursor;
pub mod feature;
pub mod storage;
pub mod tag;
pub mod layer;
#[allow(missing_docs)]
pub mod vector_tile;

/// Reads a Vector File and produces an SVG fragment for a tile.
pub fn process<R: Read>(mut r: R) -> ProtobufResult<String> {
    let tile: Tile = protobuf::parse_from_reader(&mut r)?;
    let mut storage = Storage::new();

    for raw_layer in tile.get_layers() {
        let mut layer = Layer::new(raw_layer);
        layer.paint(&mut storage)?;
    }
    Ok(String::from(storage))
}

// Only the functions listed here are callable from JS.
#[cfg_attr(target_arch="asmjs",
           link_args="-s EXPORTED_FUNCTIONS=['_process_web','_free_cstring_web']")]
extern {}

/// Process a map tile.
///
/// Takes an array as a pointer and a length.
/// Returns a C string.
#[no_mangle]
pub extern fn process_web(p: *const u8, len: usize) -> *const c_char {
    let input = unsafe {
        slice::from_raw_parts(p, len)
    };
    let output = process(input).expect("input file is valid");
    CString::new(output).expect("string contains no internal NULL").into_raw()
}

/// CString must not be just freed but the drop function must run according to docs.
///
/// Note: The Drop impl writes 0 in the byte of string and makes it unusable.
#[no_mangle]
pub extern fn free_cstring_web(p: *mut c_char) {
    unsafe {
        if p.is_null() { return }
        CString::from_raw(p)
    };
}

/// asm.js expects a main function.
fn main() {}
