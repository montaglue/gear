#![no_std]

use gcore::msg;
use gstd::prelude::*;

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let data = vec![0u8; 32768];
    msg::reply(&data, 0);
    panic!()
}
