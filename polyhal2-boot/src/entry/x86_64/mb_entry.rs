use core::{arch::global_asm, slice};

use multiboot::information::{MemoryManagement, Multiboot, PAddr};
use polyhal2_core::{bit, consts::KERNEL_OFFSET};

/// Flags set in the 'flags' member of the multiboot header.
///
/// (bits 1, 16: memory information, address fields in header)
/// bits 2 graphic information
pub(super) const MULTIBOOT_HEADER_FLAGS: u32 = bit!(1) | bit!(16) | bit!(2);

/// The magic field should contain this.
pub(super) const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BADB002;

global_asm!(
    include_str!("multiboot.S"),
    mb_hdr_magic = const MULTIBOOT_HEADER_MAGIC,
    mb_hdr_flags = const MULTIBOOT_HEADER_FLAGS,
    graphic_mode = const 0,
    offset = const (polyhal2_core::consts::KERNEL_OFFSET),
);

static mut MEM: Mem = Mem;

struct Mem;

impl MemoryManagement for Mem {
    unsafe fn paddr_to_slice(&self, addr: PAddr, size: usize) -> Option<&'static [u8]> {
        let ptr = (addr as usize | KERNEL_OFFSET) as *const u8;
        unsafe { Some(slice::from_raw_parts(ptr, size)) }
    }

    // If you only want to read fields, you can simply return `None`.
    unsafe fn allocate(&mut self, _length: usize) -> Option<(PAddr, &mut [u8])> {
        None
    }

    unsafe fn deallocate(&mut self, addr: PAddr) {
        if addr != 0 {
            unimplemented!()
        }
    }
}

/// mboot_ptr is the initial pointer to the multiboot structure
/// provided in %ebx on start-up.
#[allow(static_mut_refs)]
pub fn use_multiboot(mboot_ptr: PAddr) -> Option<Multiboot<'static, 'static>> {
    unsafe { Multiboot::from_ptr(mboot_ptr, &mut MEM) }
}
