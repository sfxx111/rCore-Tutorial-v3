// os/src/task/mod.rs

mod context;
mod task;

use crate::sync::UPSafeCell;
use lazy_static::*;
use alloc::vec::Vec;
use core::arch::global_asm;

pub use context::TaskContext;
pub use task::{TaskControlBlock, TaskStatus};

global_asm!(include_str!("switch.S"));

extern "C" {
    fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

pub struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        // 🌟 完美修复：直接调用第4章 loader 模块的公开接口，彻底抛弃老旧的汇编符号 _num_app
        let num_app = crate::loader::get_num_app();
        let mut tasks = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(crate::loader::get_app_data(i), i));
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    pub fn run_first_task(&self) {
        let mut inner = self.inner.exclusive_access();
        let next_task_cx_ptr = &inner.tasks[0].task_cx as *const TaskContext;
        inner.current_task = 0;
        inner.tasks[0].task_status = TaskStatus::Running;

        let mut _unused = TaskContext::zero_init();
        let current_task_cx_ptr = &mut _unused as *mut TaskContext;

        drop(inner);
        unsafe {
            __switch(current_task_cx_ptr, next_task_cx_ptr);
        }
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + 1 + self.num_app)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[current].task_status = TaskStatus::Ready;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.inner_run_next_task(next);
        } else {
            println!("All applications completed!");
            crate::sbi::shutdown();
        }
    }
}

impl TaskManagerInner {
    fn inner_run_next_task(&mut self, next: usize) {
        let current = self.current_task;
        self.current_task = next;
        let current_task_cx_ptr = &mut self.tasks[current].task_cx as *mut TaskContext;
        let next_task_cx_ptr = &self.tasks[next].task_cx as *const TaskContext;
        drop(self);
        unsafe {
            __switch(current_task_cx_ptr, next_task_cx_ptr);
        }
    }
}

// ======================== 全局公开接口 ========================

pub fn current_user_token() -> usize {
    let inner = TASK_MANAGER.inner.exclusive_access();
    inner.tasks[inner.current_task].get_user_token()
}

pub fn current_trap_cx() -> &'static mut crate::trap::TrapContext {
    let inner = TASK_MANAGER.inner.exclusive_access();
    inner.tasks[inner.current_task].get_trap_cx()
}

pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

pub fn run_first_task() -> ! {
    TASK_MANAGER.run_first_task();
    panic!("Unreachable in run_first_task!");
}
