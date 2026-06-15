// os/src/mm/mod.rs

pub mod address;
pub mod frame_allocator;
pub mod heap_allocator;
pub mod page_table;
pub mod memory_set;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, StepByOne};
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use page_table::{PTEFlags, PageTableEntry, PageTable, translated_byte_buffer};
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE, remap_test};

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}
