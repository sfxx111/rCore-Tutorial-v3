use super::TaskContext;
use crate::mm::{MapPermission, MemorySet, PhysPageNum, KERNEL_SPACE};
use crate::config::{TRAP_CONTEXT, kernel_stack_position};
use crate::trap::TrapContext;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // 1. 解析用户的二进制 ELF 文件，并创建独立的用户态三级页表映射空间
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);

        // 2. 查阅用户的页表映射，定位到 TRAP_CONTEXT 对应的物理页号
        let trap_cx_ppn = memory_set
            .page_table
            .translate(TRAP_CONTEXT.into())
            .unwrap()
            .ppn();

        let task_status = TaskStatus::Ready;

        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE
            .exclusive_access()
            .insert_framed_area(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
                MapPermission::R | MapPermission::W,
            );
        let kernel_sp = kernel_stack_top;
        
        let task_cx = TaskContext::goto_trap_return(kernel_sp);

        let tcb = Self {
            task_status,
            task_cx,
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };

        // 4. 在刚刚映射的物理页帧上，写入定制的 TrapContext 初始值
        let trap_cx = tcb.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_sp,
            crate::trap::trap_handler as usize,
        );

        tcb
    }

    // 通过物理页号直接计算内核中的裸指针
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        let paddr: usize = self.trap_cx_ppn.0 << 12;
        unsafe { (paddr as *mut TrapContext).as_mut().unwrap() }
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
}
