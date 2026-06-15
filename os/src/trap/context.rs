use riscv::register::sstatus::{Sstatus, self, SPP};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    // ======= 🌟 以下是本章实验新增的内核控制信息 =======
    pub kernel_satp: usize,   // 内核页表的 token (satp 寄存器的值)
    pub kernel_sp: usize,     // 当前应用对应的内核栈顶虚拟地址
    pub trap_handler: usize,  // 内核中断处理函数 trap_handler 的入口虚拟地址
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) { self.x[2] = sp; }
    
    // ======= 🌟 升级后的初始化函数，用于在 TCB 中给新应用初始化上下文 =======
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User); // 确保 sret 后回到用户态
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_satp,   // 写入内核页表
            kernel_sp,     // 写入内核栈顶
            trap_handler,  // 写入处理函数入口
        };
        cx.set_sp(sp); // 设置用户态的栈指针
        cx
    }
}

