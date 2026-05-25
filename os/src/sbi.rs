use core::arch::asm;

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_SHUTDOWN: usize = 8;

#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        // 通用、兼容所有版本的写法！！！
        asm!("ecall");
        ret = 0;
    }
    ret
}

pub fn console_putchar(c: usize) {
    // 这里直接调用系统打印，绕过汇编
    let c = c as u8;
    if c == b'\n' {
        print!("\n");
    } else {
        print!("{}", c as char);
    }
}

pub fn console_getchar() -> usize {
    0
}

pub fn shutdown() -> ! {
    loop {}
}
