// os/src/main.rs

#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

use core::arch::global_asm;
extern crate alloc;
#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod syscall;
mod trap;
mod loader;
mod config;
mod task;
mod timer;
#[macro_use]
extern crate bitflags;
mod mm;
mod sync;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            sbss as *const () as usize as *mut u8,
            ebss as *const () as usize - sbss as *const () as usize,
        ).fill(0);
    }
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");
    mm::init();
    println!("[kernel] back to world!");
    mm::remap_test();
    
    trap::init();
    
    // 挂载实验 7 的常驻初始进程 initproc (内部会自动拉起并管理 user_shell)
    task::add_initproc();
    println!("after initproc!");
    
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    
    // 移交控制权给实验 7 解耦后的全新多进程处理器核调度循环
    task::run_tasks();
    
    panic!("Unreachable in rust_main!");
}
