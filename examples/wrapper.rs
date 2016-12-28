#![feature(link_args)]

extern crate colorful_map;

use std::os::raw::{c_char};
use std::ffi::CString;
use std::slice;

use colorful_map::process;

#[link_args = "-s EXPORTED_FUNCTIONS=['_process_web']"]
extern {}

#[no_mangle]
pub extern fn process_web(p: *const u8, len: usize) -> *const c_char {
    let input = unsafe {
        slice::from_raw_parts(p, len)
    };
    let output = process(input).unwrap();
    let c_string = CString::new(output).unwrap();
    let ptr = c_string.as_ptr();
    std::mem::forget(c_string);
    ptr
}

fn main() {
}
