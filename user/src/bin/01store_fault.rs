#![no_std]
#![no_main]
#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Into Test store_fault...");
    println!("Kernel should kill this app!");
    unsafe { (0x0 as *mut u8).write_volatile(0); }
    0
}
