// os/src/task/mod.rs
mod context;
mod pid;
mod task;
mod manager;
mod processor;

use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;
use lazy_static::*;

pub use context::TaskContext;
pub use task::{TaskControlBlock, TaskStatus};
pub use manager::add_task;
pub use processor::{
    run_tasks, current_task, current_user_token, current_trap_cx, take_current_task
};

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    
    add_task(task);
    let mut processor = PROCESSOR.lock();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    
    unsafe { __switch(task_cx_ptr, idle_task_cx_ptr); }
}

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();
    let mut shortcut = task.inner_exclusive_access();
    shortcut.task_status = TaskStatus::Zombie;
    shortcut.exit_code = exit_code;
    
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in shortcut.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    shortcut.children.clear();
    shortcut.memory_set.recycle_data_pages();
    drop(shortcut);
    drop(task);
    
    let mut processor = PROCESSOR.lock();
    let mut _unused = TaskContext::zero_init();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    
    unsafe { __switch(&mut _unused as *mut TaskContext, idle_task_cx_ptr); }
}

use processor::PROCESSOR;
extern "C" {
    fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
