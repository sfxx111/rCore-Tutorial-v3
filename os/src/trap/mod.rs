mod context;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    stvec,
    scause::{self, Trap, Exception},
    stval,
};
use crate::syscall::syscall;
//use crate::batch::run_next_app;

global_asm!(include_str!("trap.S"));

pub fn init() {
    unsafe extern "C" { 
        // 显式声明外部汇编符号
        fn __alltraps(); 
    }
    unsafe { stvec::write(__alltraps as *const () as usize, TrapMode::Direct); }
}

// 使用符合最新规范的外部 C 导出，确保汇编能够无障碍调用到此 C 函数
#[unsafe(no_mangle)]
pub unsafe extern "C" fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, core dumped.");
//            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
//            run_next_app();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}

pub use context::TrapContext;
