// os/src/task/context.rs

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    
    // 强制将内核任务切换的返回目标对准跳板中的 trap_return 汇编
    pub fn goto_trap_return(kernel_sp: usize) -> Self {
        extern "C" {
            fn trap_return();
        }
        Self {
            ra: trap_return as usize,
            sp: kernel_sp,
            s: [0; 12],
        }
    }
}
