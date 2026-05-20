#![no_std]
#![no_main]

#[macro_use]
mod console;
mod lang_items;
mod sbi;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    unsafe extern "C" {
        static mut sbss: u8;
        static mut ebss: u8;
    }
    let start = unsafe { &mut sbss as *mut u8 as usize };
    let end = unsafe { &mut ebss as *mut u8 as usize };
    unsafe {
        core::slice::from_raw_parts_mut(start as *mut u8, end - start).fill(0);
    }
}

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    unsafe extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack();
        fn boot_stack_top();
    }

    clear_bss();

    println!("Hello, RISC-V Kernel!");
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);

    panic!("Shutdown now!");
}
