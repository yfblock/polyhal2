use bitflags::bitflags;
use polyhal2_core::{
    addr::{PhysAddr, VirtAddr},
    bit,
};
use riscv::{asm::sfence_vma, register::satp};

use crate::{MappingFlags, MappingSize, PTE, TLB, VSpace};

impl PTE {
    #[inline]
    pub const fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate((self.0 & 0xff) as u64)
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V) && self.0 > u8::MAX as usize
    }

    #[inline]
    pub(crate) fn is_table(&self) -> bool {
        self.flags().contains(PTEFlags::V)
            && !(self.flags().contains(PTEFlags::R)
                || self.flags().contains(PTEFlags::W)
                || self.flags().contains(PTEFlags::X))
    }

    #[inline]
    pub(crate) const fn new_table(paddr: PhysAddr) -> Self {
        Self((paddr.raw() >> 2) | (PTEFlags::V).bits() as usize)
    }

    #[inline]
    pub(crate) const fn new_page(paddr: PhysAddr, flags: PTEFlags, _: MappingSize) -> Self {
        Self((paddr.raw() >> 2) | flags.bits() as usize)
    }

    #[inline]
    pub(crate) const fn paddr(&self) -> PhysAddr {
        PhysAddr::new(self.0 << 2).floor(VSpace::PAGE_SIZE)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PTEFlags: u64 {
        const V = bit!(0);
        const R = bit!(1);
        const W = bit!(2);
        const X = bit!(3);
        const U = bit!(4);
        const G = bit!(5);
        const A = bit!(6);
        const D = bit!(7);

        const VRWX  = Self::V.bits() | Self::R.bits() | Self::W.bits() | Self::X.bits();
        const ADUVRX = Self::A.bits() | Self::D.bits() | Self::U.bits() | Self::V.bits() | Self::R.bits() | Self::X.bits();
        const ADVRWX = Self::A.bits() | Self::D.bits() | Self::VRWX.bits();
        const ADGVRWX = Self::G.bits() | Self::ADVRWX.bits();
    }
}

impl From<MappingFlags> for PTEFlags {
    fn from(flags: MappingFlags) -> Self {
        if flags.is_empty() {
            Self::empty()
        } else {
            let mut res = Self::V;
            if flags.contains(MappingFlags::R) {
                res |= PTEFlags::R | PTEFlags::A;
            }
            if flags.contains(MappingFlags::W) {
                res |= PTEFlags::W | PTEFlags::D;
            }
            if flags.contains(MappingFlags::X) {
                res |= PTEFlags::X;
            }
            if flags.contains(MappingFlags::U) {
                res |= PTEFlags::U;
            }
            res
        }
    }
}

impl From<PTEFlags> for MappingFlags {
    fn from(value: PTEFlags) -> Self {
        let mut mapping_flags = MappingFlags::empty();
        if value.contains(PTEFlags::V) {
            mapping_flags |= MappingFlags::P;
        }
        if value.contains(PTEFlags::R) {
            mapping_flags |= MappingFlags::R;
        }
        if value.contains(PTEFlags::W) {
            mapping_flags |= MappingFlags::W;
        }
        if value.contains(PTEFlags::X) {
            mapping_flags |= MappingFlags::X;
        }
        if value.contains(PTEFlags::U) {
            mapping_flags |= MappingFlags::U;
        }
        if value.contains(PTEFlags::A) {
            mapping_flags |= MappingFlags::A;
        }
        if value.contains(PTEFlags::D) {
            mapping_flags |= MappingFlags::D;
        }

        mapping_flags
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
        Self(PhysAddr::new(satp::read().ppn() << 12))
    }

    /// Change the pagetable to Virtual space.
    #[inline]
    pub fn switch(&self) {
        // Write page table entry for
        satp::write((8 << 60) | (self.0.raw() >> 12));
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
            sfence_vma(vaddr.raw(), 0);
        }
    }

    /// flush all tlb entry
    ///
    /// how to use ?
    /// just
    /// TLB::flush_all();
    #[inline]
    pub fn flush_all() {
        riscv::asm::sfence_vma_all();
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
