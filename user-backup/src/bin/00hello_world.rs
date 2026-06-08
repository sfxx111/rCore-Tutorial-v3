#![no_std]
#![no_main]
use core::arch::asm;
#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Hello, world!");
    unsafe { asm!("sret"); } // 故意在U态使用内核指令，用以测试特权级非法指令抓取
    0
}
