// os/src/sync.rs

use core::cell::{RefCell, RefMut};

/// 专为单核（Uniprocessor）环境设计的安全单元
/// 它通过 RefCell 保证独占访问，并实现 Sync 绕过编译器的多线程安全检查
pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// 安全地包装一个全局变量
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    
    /// 独占访问内部数据，若已被借用则会触发 panic
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
