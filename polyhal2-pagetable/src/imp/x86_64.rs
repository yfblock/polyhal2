use bitflags::bitflags;
use polyhal2_core::{
    addr::{PhysAddr, VirtAddr},
    bit,
};
use x86_64::{instructions::tlb, registers::control::Cr3};

use crate::{MappingFlags, MappingSize, PTE, TLB, VSpace};

bitflags! {
    pub struct PTEFlags: u64 {
        /// Page is present in the page table
        const P         = bit!(0);
        /// Read/Write; if 0, Only read
        const RW        = bit!(1);
        /// User/Supervisor; if 0, Only supervisor
        const US        = bit!(2);
        /// Page-level wright-through
        const PWT       = bit!(3);
        /// Page-level cache disable.
        const PCD       = bit!(4);
        /// Accessed; indicates whether software has accessed the 4-KByte page
        const A         = bit!(5);
        /// Dirty; indicates whether software has written to the 4-KByte page referenced by this entry.
        const D         = bit!(6);
        /// Page size; if set this entry maps a 2-MByte page; otherwise, this entry references a page directory.
        const PS      = bit!(7);
        /// Global; if CR4.PGE = 1, determines whether the translation is global (see Section 4.10); ignored otherwise
        const G         = bit!(8);
        /// User defined flag -- ignored by hardware (bit 9)
        const USER_9    = bit!(9);
        /// User defined flag -- ignored by hardware (bit 10)
        const USER_10   = bit!(10);
        /// User defined flag -- ignored by hardware (bit 11)
        const USER_11   = bit!(11);
        ///  If IA32_EFER.NXE = 1, execute-disable
        ///  If 1, instruction fetches are not allowed from the 512-GByte region.
        const XD        = bit!(63);
    }
}

impl From<MappingFlags> for PTEFlags {
    fn from(flags: MappingFlags) -> Self {
        let mut res = Self::P;
        if flags.contains(MappingFlags::W) {
            res |= Self::RW;
        }
        if flags.contains(MappingFlags::U) {
            res |= Self::US;
        }
        if flags.contains(MappingFlags::A) {
            res |= Self::A;
        }
        if flags.contains(MappingFlags::D) {
            res |= Self::D;
        }
        if flags.contains(MappingFlags::X) {
            res.remove(Self::XD);
        }
        res
    }
}

impl From<PTEFlags> for MappingFlags {
    fn from(value: PTEFlags) -> Self {
        let mut res = MappingFlags::empty();
        if value.contains(PTEFlags::RW) {
            res |= MappingFlags::W;
        }
        if value.contains(PTEFlags::US) {
            res |= MappingFlags::U;
        }
        if value.contains(PTEFlags::A) {
            res |= MappingFlags::A;
        }
        if value.contains(PTEFlags::D) {
            res |= MappingFlags::D;
        }
        if value.contains(PTEFlags::XD) {
            res |= MappingFlags::X;
        }
        res
    }
}

impl PTE {
    #[inline]
    pub(crate) fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::P)
    }

    #[inline]
    pub(crate) fn is_table(&self) -> bool {
        self.flags().contains(PTEFlags::P) & !self.flags().contains(PTEFlags::PS)
    }

    #[inline]
    pub(crate) fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0 as _)
    }

    #[inline]
    pub(crate) fn new_table(paddr: PhysAddr) -> Self {
        Self(paddr.raw() | (PTEFlags::P | PTEFlags::US | PTEFlags::RW).bits() as usize)
    }

    #[inline]
    pub(crate) fn new_page(paddr: PhysAddr, flags: PTEFlags, size: MappingSize) -> Self {
        match size {
            MappingSize::Page4KB => Self(paddr.raw() | flags.bits() as usize),
            MappingSize::Page2MB => todo!(),
            MappingSize::Page1GB => todo!(),
        }
    }

    #[inline]
    pub(crate) fn paddr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0xFFFF_FFFF_F000)
    }
}

impl VSpace {
    /// The size of the page for this platform.
    pub const PAGE_SIZE: usize = 0x1000;
    /// The stages of the address translation
    pub const PAGE_LEVEL: usize = 4;
    pub(crate) const PTE_NUM_IN_PAGE: usize = 0x200;
    pub(crate) const GLOBAL_ROOT_PTE_RANGE: usize = 0x100;

    /// Get the using PageTable currently.
    #[inline]
    pub fn current() -> Self {
        Self(PhysAddr::new(
            Cr3::read().0.start_address().as_u64() as usize
        ))
    }

    /// Change the pagetable to Virtual space.
    #[inline]
    pub fn change(&self) {
        unsafe {
            core::arch::asm!("mov cr3, {}", in(reg) self.0.raw());
        }
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
        tlb::flush(x86_64::VirtAddr::new_truncate(vaddr.raw() as _))
    }

    /// flush all tlb entry
    ///
    /// how to use ?
    /// just
    /// TLB::flush_all();
    #[inline]
    pub fn flush_all() {
        tlb::flush_all()
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
