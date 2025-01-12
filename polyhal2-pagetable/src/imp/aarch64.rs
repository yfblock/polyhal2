use core::marker::PhantomData;

use aarch64_cpu::registers::{TTBR0_EL1, Writeable};
use polyhal2_core::{
    addr::{PhysAddr, VirtAddr},
    bit,
    consts::PAGE_SIZE,
};

use crate::{MappingFlags, PTE, TLB, VSpace, VSpaceAO};

impl PTE {
    #[inline]
    pub const fn paddr(&self) -> PhysAddr {
        PhysAddr::new(self.0).floor(PAGE_SIZE)
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn set(&mut self, ppn: usize, flags: PTEFlags) {
        self.0 = (ppn << 10) | flags.bits();
    }

    #[inline]
    pub const fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::VALID)
    }

    #[inline]
    pub fn is_table(&self) -> bool {
        self.flags().contains(PTEFlags::NON_BLOCK | PTEFlags::VALID)
    }

    #[inline]
    pub(crate) fn new_table(paddr: PhysAddr) -> Self {
        Self(paddr.raw() | 0b11)
    }

    /// Create a new PageTableEntry from ppn and flags
    pub const fn new_page(paddr: PhysAddr, flags: PTEFlags) -> Self {
        Self(paddr.raw() | flags.bits())
    }
}

impl From<MappingFlags> for PTEFlags {
    fn from(value: MappingFlags) -> Self {
        let mut flags = PTEFlags::VALID | PTEFlags::NON_BLOCK | PTEFlags::AF;
        if !value.contains(MappingFlags::W) {
            flags |= PTEFlags::AP_RO;
        }

        if !value.contains(MappingFlags::X) {
            flags |= PTEFlags::UXN | PTEFlags::PXN;
        }

        if value.contains(MappingFlags::U) {
            flags |= PTEFlags::AP_EL0;
        }
        if !value.contains(MappingFlags::G) {
            flags |= PTEFlags::NG
        }
        flags
    }
}

impl From<PTEFlags> for MappingFlags {
    fn from(value: PTEFlags) -> Self {
        if value.is_empty() {
            return MappingFlags::empty();
        };
        let mut flags = MappingFlags::R;

        if !value.contains(PTEFlags::AP_RO) {
            flags |= MappingFlags::W;
        }
        if !value.contains(PTEFlags::UXN) || !value.contains(PTEFlags::PXN) {
            flags |= MappingFlags::X;
        }
        if value.contains(PTEFlags::AP_EL0) {
            flags |= MappingFlags::U;
        }
        if value.contains(PTEFlags::AF) {
            flags |= MappingFlags::A;
        }
        if !value.contains(PTEFlags::NG) {
            flags |= MappingFlags::G;
        }
        flags
    }
}

bitflags::bitflags! {
    /// Possible flags for a page table entry.
    pub struct PTEFlags: usize {
        // Attribute fields in stage 1 VMSAv8-64 Block and Page descriptors:
        /// Whether the descriptor is valid.
        const VALID =       bit!(0);
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   bit!(1);
        /// Memory attributes index field.
        const ATTR_INDX =   0b111 << 2;
        ///
        const NORMAL_NONCACHE = 0b010 << 2;
        /// Non-secure bit. For memory accesses from Secure state, specifies whether the output
        /// address is in Secure or Non-secure memory.
        const NS =          bit!(5);
        /// Access permission: accessable at EL0.
        const AP_EL0 =      bit!(6);
        /// Access permission: read-only.
        const AP_RO =       bit!(7);
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER =       bit!(8);
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   bit!(9);
        /// The Access flag.
        const AF =          bit!(10);
        /// The not global bit.
        const NG =          bit!(11);
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  bit!(52);
        /// The Privileged execute-never field.
        const PXN =         bit!(53);
        /// The Execute-never or Unprivileged execute-never field.
        const UXN =         bit!(54);

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors:

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           bit!(59);
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            bit!(60);
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     bit!(61);
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   bit!(62);
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            bit!(63);
    }
}

impl<T: VSpaceAO> VSpace<T> {
    /// The size of the page for this platform.
    pub(crate) const PAGE_SIZE: usize = 0x1000;
    pub(crate) const PAGE_LEVEL: usize = 3;
    pub(crate) const PTE_NUM_IN_PAGE: usize = 0x200;
    pub(crate) const GLOBAL_ROOT_PTE_RANGE: usize = 0x200;
    pub(crate) const VADDR_BITS: usize = 39;
    pub(crate) const USER_VADDR_END: usize = (1 << Self::VADDR_BITS) - 1;

    /// Get the using PageTable currently.
    #[inline]
    pub fn current() -> Self {
        Self(PhysAddr::new(TTBR0_EL1.get_baddr() as _), PhantomData)
    }

    /// Change the pagetable to Virtual space.
    #[inline]
    pub fn switch(&self) {
        TTBR0_EL1.set(self.0.floor(Self::PAGE_SIZE).raw() as _);
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
            core::arch::asm!(
                "
                    tlbi vaale1is, {}
                    dsb sy
                    isb
                ", 
                in(reg) ((vaddr.raw() >> 12) & 0xFFFF_FFFF_FFFF)
            )
        }
    }

    /// flush all tlb entry
    ///
    /// TLB::flush_all();
    #[inline]
    pub fn flush_all() {
        unsafe { core::arch::asm!("tlbi vmalle1; dsb sy; isb") }
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
