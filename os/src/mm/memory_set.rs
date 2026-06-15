// os/src/mm/memory_set.rs 

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::arch::asm;
use riscv::register::satp;
use lazy_static::*;

use bitflags::bitflags;

use super::{PageTable, PTEFlags};
use super::{VirtPageNum, VirtAddr, PhysPageNum, PhysAddr};
use crate::mm::address::{VPNRange, StepByOne};
use super::{frame_alloc, FrameTracker};

use crate::config::{PAGE_SIZE, MEMORY_END, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use crate::sync::UPSafeCell;

bitflags! {
    pub struct MapPermission: u8 { 
        const R = 1 << 1; 
        const W = 1 << 2; 
        const X = 1 << 3; 
        const U = 1 << 4; 
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType { Identical, Framed }

pub struct MapArea { pub vpn_range: VPNRange, pub data_frames: BTreeMap<VirtPageNum, FrameTracker>, pub map_type: MapType, pub map_perm: MapPermission, }
impl MapArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, map_perm: MapPermission) -> Self {
        Self { vpn_range: VPNRange::new(start_va.floor(), end_va.ceil()), data_frames: BTreeMap::new(), map_type, map_perm }
    }
    pub fn map(&mut self, page_table: &mut PageTable) { for vpn in self.vpn_range { self.map_one(page_table, vpn); } }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn = match self.map_type {
            MapType::Identical => PhysPageNum(vpn.0),
            MapType::Framed => { let frame = frame_alloc().unwrap(); let p = frame.ppn; self.data_frames.insert(vpn, frame); p }
        };
        page_table.map(vpn, ppn, PTEFlags::from_bits(self.map_perm.bits).unwrap());
    }
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        let mut start: usize = 0; let mut current_vpn = self.vpn_range.get_start();
        while start < data.len() {
            let src = &data[start..data.len().min(start + PAGE_SIZE)];
            let pte = page_table.translate(current_vpn).unwrap();
            pte.ppn().get_bytes_array()[..src.len()].copy_from_slice(src);
            start += PAGE_SIZE; current_vpn.step();
        }
    }
}

pub struct MemorySet { pub page_table: PageTable, pub areas: Vec<MapArea>, }
impl MemorySet {
    pub fn new_bare() -> Self { Self { page_table: PageTable::new(), areas: Vec::new() } }
    pub fn token(&self) -> usize { self.page_table.token() }
    pub fn activate(&self) { unsafe { satp::write(self.page_table.token()); asm!("sfence.vma"); } }
    pub fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data { map_area.copy_data(&mut self.page_table, data); }
        self.areas.push(map_area);
    }
    pub fn insert_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) {
        self.push(MapArea::new(start_va, end_va, MapType::Framed, permission), None);
    }
    fn map_trampoline(&mut self) {
        extern "C" { fn strampoline(); }
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as *const () as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    pub fn new_kernel() -> Self {
        extern "C" { fn stext(); fn etext(); fn srodata(); fn erodata(); fn sdata(); fn edata(); fn sbss_with_stack(); fn ebss(); fn ekernel(); }
        let mut memory_set = Self::new_bare();
        memory_set.map_trampoline();
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(".bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
        println!("mapping .text section");
        memory_set.push(MapArea::new((stext as usize).into(), (etext as usize).into(), MapType::Identical, MapPermission::R | MapPermission::X), None);
        println!("mapping .rodata section");
        memory_set.push(MapArea::new((srodata as usize).into(), (erodata as usize).into(), MapType::Identical, MapPermission::R), None);
        println!("mapping .data section");
        memory_set.push(MapArea::new((sdata as usize).into(), (edata as usize).into(), MapType::Identical, MapPermission::R | MapPermission::W), None);
        println!("mapping .bss section");
        memory_set.push(MapArea::new((sbss_with_stack as usize).into(), (ebss as usize).into(), MapType::Identical, MapPermission::R | MapPermission::W), None);
        println!("mapping physical memory");
        memory_set.push(MapArea::new((ekernel as usize).into(), MEMORY_END.into(), MapType::Identical, MapPermission::R | MapPermission::W), None);
        memory_set
    }
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        memory_set.map_trampoline();
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() { map_perm |= MapPermission::R; }
                if ph_flags.is_write() { map_perm |= MapPermission::W; }
                if ph_flags.is_execute() { map_perm |= MapPermission::X; }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(map_area, Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]));
            }
        }
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(MapArea::new(user_stack_bottom.into(), user_stack_top.into(), MapType::Framed, MapPermission::R | MapPermission::W | MapPermission::U), None);
        memory_set.push(MapArea::new(TRAP_CONTEXT.into(), TRAMPOLINE.into(), MapType::Framed, MapPermission::R | MapPermission::W), None);
        (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
    }
}

#[allow(unused)]
pub fn remap_test() {
    extern "C" { fn stext(); fn etext(); fn srodata(); fn erodata(); fn sdata(); fn edata(); }
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space.page_table.translate(mid_text.floor()).unwrap().writable(),
        false
    );
    assert_eq!(
        kernel_space.page_table.translate(mid_rodata.floor()).unwrap().writable(),
        false,
    );
    assert_eq!(
        kernel_space.page_table.translate(mid_data.floor()).unwrap().executable(),
        false,
    );
    println!("remap_test passed!");
}

lazy_static! {
    pub static ref KERNEL_SPACE: UPSafeCell<MemorySet> = unsafe {
        UPSafeCell::new(MemorySet::new_kernel())
    };
}
