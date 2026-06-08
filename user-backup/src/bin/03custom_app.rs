#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("====================================================");
    println!("Hello! This is my own custom application!");
    println!("I am running in User Mode (U-Mode) securely.");
    println!("Now, I will try to trigger an explicit panic...");
    println!("====================================================");
    
    // 显式触发终止，用以测试内核对应用异常的捕获与批处理流的推进
    panic!("Intentional Custom App Panic!"); 
    0
}
