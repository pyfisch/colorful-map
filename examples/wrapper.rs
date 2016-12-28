#![feature(link_args)]

extern crate colorful_map;

use std::os::raw::{c_char};
use std::ffi::CString;
use std::slice;

use colorful_map::process;

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

fn main() {
}
