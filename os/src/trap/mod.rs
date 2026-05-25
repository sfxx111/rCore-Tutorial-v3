use core::arch::global_asm;
use riscv::register::{stvec, scause::{Trap, Exception}, stval};

global_asm!(include_str!("trap.S"));

mod context;
pub use context::TrapContext;
use crate::syscall::syscall;
use crate::batch::run_next_app;

pub fn init() {
    extern "C" { fn __alltraps(); }
    unsafe { stvec::write(__alltraps as usize, stvec::TrapMode::Direct); }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let cause = riscv::register::scause::read().cause();
    let stv = stval::read();
    match cause {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault, kill app");
            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction, kill app");
            run_next_app();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval={:#x}", cause, stv);
        }
    }
    cx
}
