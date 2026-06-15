// os/src/trap/mod.rs

mod context;

use core::arch::global_asm;
use riscv::register::{
    stvec,
    sie,
    scause::{self, Trap, Exception, Interrupt},
    stval,
};
use crate::syscall::syscall;
use crate::task::{current_user_token, current_trap_cx, exit_current_and_run_next, suspend_current_and_run_next};
use crate::config::TRAMPOLINE;
use crate::timer::set_next_trigger;

global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kernel_trap_entry();
}

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, stvec::TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    
    let cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) 
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] PageFault in application, core dumped.");
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = crate::config::TRAP_CONTEXT;
    let user_token = current_user_token();
    
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    
    unsafe {
        core::arch::asm!(
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_token,
            options(noreturn)
        );
    }
}

pub use context::TrapContext;
