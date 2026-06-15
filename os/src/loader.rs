// os/src/loader.rs

/// 获取解压出来的应用程序总数
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as *const usize).read_volatile() }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as *const usize;
    let num_app = unsafe { num_app_ptr.read_volatile() };
    assert!(app_id < num_app);

    // 通过指针偏移从符号表中查找对应 ELF 文件的起始和结束物理地址
    let app_start_ptr = unsafe { num_app_ptr.add(num_app + 1 + app_id) as *const usize };
    let start = unsafe { app_start_ptr.read_volatile() };
    let end = unsafe { app_start_ptr.add(1).read_volatile() };

    unsafe { core::slice::from_raw_parts(start as *const u8, end - start) }
}
