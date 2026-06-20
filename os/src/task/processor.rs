// os/src/task/processor.rs
use super::{TaskControlBlock, TaskContext, TaskStatus, fetch_task, add_task};
use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self { current: None, idle_task_cx: TaskContext::zero_init() }
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
    pub fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
}

lazy_static! {
    pub static ref PROCESSOR: Mutex<Processor> = Mutex::new(Processor::new());
}

pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.lock();
        if let Some(task) = fetch_task() {
            let mut task_inner = task.inner_exclusive_access();
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            task_inner.task_status = TaskStatus::Running;
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            drop(task_inner);
            processor.current = Some(task);
            drop(processor);
           
            unsafe { __switch(idle_task_cx_ptr, next_task_cx_ptr); }
        }
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut crate::trap::TrapContext {
    current_task().unwrap().inner_exclusive_access().get_trap_cx()
}

extern "C" {
    fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
