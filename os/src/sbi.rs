use core::arch::asm;

#[inline(always)]
fn sbi_call(extension: usize, function: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            in("a0") arg0,
            in("a1") arg1,
            in("a2") arg2,
            in("a6") function,
            in("a7") extension,
            lateout("a0") ret,
        );
    }
    ret
}

pub fn console_putchar(c: usize) {
    sbi_call(1, 0, c, 0, 0);
}

pub fn set_timer(timer: usize) {
    sbi_call(0x54494D45, 0, timer, 0, 0);
}

pub fn shutdown() -> ! {
    sbi_call(0x53525354, 0, 0, 0, 0);
    panic!("It should shutdown!");
}
