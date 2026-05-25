use core::arch::asm;
use core::cell::RefCell;
use lazy_static::lazy_static;
use crate::trap::TrapContext;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack { data: [0; KERNEL_STACK_SIZE] };
static USER_STACK: UserStack = UserStack { data: [0; USER_STACK_SIZE] };

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let p = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe { *p = cx; p.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManagerInner {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

struct AppManager {
    inner: RefCell<AppManagerInner>,
}

unsafe impl Sync for AppManager {}

lazy_static! {
    static ref APP_MANAGER: AppManager = AppManager {
        inner: RefCell::new({
            extern "C" { fn _num_app(); }
            let ptr = _num_app as *const usize;
            let n = unsafe { *ptr };
            let mut arr = [0; MAX_APP_NUM+1];
            let v = unsafe { core::slice::from_raw_parts(ptr.add(1), n+1) };
            arr[..=n].copy_from_slice(v);
            AppManagerInner {
                num_app: n,
                current_app: 0,
                app_start: arr,
            }
        })
    };
}

impl AppManagerInner {
    pub fn print_info(&self) {
        println!("[kernel] num_app={}", self.num_app);
        for i in 0..self.num_app {
            println!("[kernel] app_{} [{:#x}, {:#x})", i, self.app_start[i], self.app_start[i+1]);
        }
    }

    unsafe fn load_app(&self, id: usize) {
        if id >= self.num_app { panic!("All done!"); }
        println!("[kernel] Loading app_{}", id);
        asm!("fence.i");
        (APP_BASE_ADDRESS..APP_BASE_ADDRESS+APP_SIZE_LIMIT).for_each(|a| (a as *mut u8).write_volatile(0));
        let src = core::slice::from_raw_parts(self.app_start[id] as *const u8, self.app_start[id+1]-self.app_start[id]);
        let dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, src.len());
        dst.copy_from_slice(src);
    }
}

pub fn init() {
    APP_MANAGER.inner.borrow().print_info();
}

pub fn run_next_app() -> ! {
    let cur = APP_MANAGER.inner.borrow().current_app;
    unsafe { APP_MANAGER.inner.borrow().load_app(cur); }
    APP_MANAGER.inner.borrow_mut().current_app += 1;
    extern "C" { fn __restore(_: usize); }
    let cx = TrapContext::app_init_context(APP_BASE_ADDRESS, USER_STACK.get_sp());
    let p = KERNEL_STACK.push_context(cx);
    unsafe { __restore(p as *mut TrapContext as usize); }
    loop {}
}
