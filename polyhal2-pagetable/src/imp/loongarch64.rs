use loongArch64::register::pgdl;
use polyhal2_core::{
    addr::{PhysAddr, VirtAddr},
    bit,
    consts::PAGE_SIZE,
};

use crate::{MappingFlags, MappingSize, PTE, TLB, VSpace};

impl PTE {
    #[inline]
    pub(crate) const fn is_valid(&self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub(crate) const fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    #[inline]
    pub(crate) const fn paddr(&self) -> PhysAddr {
        PhysAddr::new(self.0).floor(PAGE_SIZE)
    }

    #[inline]
    pub(crate) const fn is_table(&self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub(crate) const fn new_table(paddr: PhysAddr) -> Self {
        Self(paddr.raw())
    }

    #[inline]
    pub(crate) const fn new_page(paddr: PhysAddr, flags: PTEFlags, size: MappingSize) -> Self {
        match size {
            MappingSize::Page4KB => Self(paddr.raw() | flags.bits()),
            MappingSize::Page2MB | MappingSize::Page1GB => panic!("Unsupported page size"),
        }
    }
}

impl From<MappingFlags> for PTEFlags {
    fn from(value: MappingFlags) -> Self {
        let mut flags = PTEFlags::V;
        if value.contains(MappingFlags::W) {
            flags |= PTEFlags::W | PTEFlags::D;
        }

        // if !value.contains(MappingFlags::X) {
        //     flags |= PTEFlags::NX;
        // }

        if value.contains(MappingFlags::U) {
            flags |= PTEFlags::PLV_USER;
        }
        flags
    }
}

impl From<PTEFlags> for MappingFlags {
    fn from(val: PTEFlags) -> Self {
        let mut flags = MappingFlags::empty();
        if val.contains(PTEFlags::W) {
            flags |= MappingFlags::W;
        }

        if val.contains(PTEFlags::D) {
            flags |= MappingFlags::D;
        }

        // if !self.contains(PTEFlags::NX) {
        //     flags |= MappingFlags::X;
        // }

        if val.contains(PTEFlags::PLV_USER) {
            flags |= MappingFlags::U;
        }
        flags
    }
}

bitflags::bitflags! {
    /// Possible flags for a page table entry.
    pub struct PTEFlags: usize {
        /// Page Valid
        const V = bit!(0);
        /// Dirty, The page has been writed.
        const D = bit!(1);

        const PLV_USER = 0b11 << 2;

        const MAT_NOCACHE = 0b01 << 4;

        /// Designates a global mapping OR Whether the page is huge page.
        const GH = bit!(6);

        /// Page is existing.
        const P = bit!(7);
        /// Page is writeable.
        const W = bit!(8);
        /// Is a Global Page if using huge page(GH bit).
        const G = bit!(10);
        /// Page is not readable.
        const NR = bit!(11);
        /// Page is not executable.
        /// FIXME: Is it just for a huge page?
        /// Linux related url: https://github.com/torvalds/linux/blob/master/arch/loongarch/include/asm/pgtable-bits.h
        const NX = bit!(12);
        /// Whether the privilege Level is restricted. When RPLV is 0, the PTE
        /// can be accessed by any program with privilege Level highter than PLV.
        const RPLV = bit!(63);
    }
}

impl VSpace {
    /// The size of the page for this platform.
    pub const PAGE_SIZE: usize = 0x1000;
    /// The stages of the address translation
    pub const PAGE_LEVEL: usize = 3;
    pub(crate) const PTE_NUM_IN_PAGE: usize = 0x200;
    pub(crate) const GLOBAL_ROOT_PTE_RANGE: usize = 0x100;

    /// Get the using PageTable currently.
    #[inline]
    pub fn current() -> Self {
        Self(PhysAddr::new(pgdl::read().base()))
    }

    /// Change the pagetable to Virtual space.
    #[inline]
    pub fn switch(&self) {
        pgdl::set_base(self.0.floor(Self::PAGE_SIZE).raw());
        TLB::flush_all();
    }
}

/// TLB operations
impl TLB {
    /// flush the TLB entry by VirtualAddress
    /// just use it directly
    ///
    /// TLB::flush_vaddr(arg0); // arg0 is the virtual address(VirtAddr)
    #[inline]
    pub fn flush_vaddr(vaddr: VirtAddr) {
        unsafe {
            core::arch::asm!("dbar 0; invtlb 0x05, $r0, {reg}", reg = in(reg) vaddr.raw());
        }
    }

    /// flush all tlb entry
    ///
    /// how to use ?
    /// just
    /// TLB::flush_all();
    #[inline]
    pub fn flush_all() {
        unsafe {
            core::arch::asm!("dbar 0; invtlb 0x00, $r0, $r0");
        }
    }
}

/// Get n level page table index of the given virtual address
#[inline]
pub const fn pg_index(vaddr: VirtAddr, n: usize) -> usize {
    (vaddr.raw() >> (12 + 9 * n)) & 0x1ff
}

/// Get n level page table offset of the given virtual address
#[inline]
pub const fn pg_offest(vaddr: VirtAddr, n: usize) -> usize {
    vaddr.raw() % (1 << (12 + 9 * n))
}
