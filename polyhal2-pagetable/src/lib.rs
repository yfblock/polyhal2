//! Make Support For Multi PlatForm
//!
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]

/// PageTable for aarch64
#[cfg_attr(target_arch = "aarch64", path = "imp/aarch64.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "imp/loongarch64.rs")]
mod imp;

use imp::{pg_index, pg_offest};
use polyhal2_core::{
    addr::{PhysAddr, VirtAddr},
    bit,
};

// static mut PAGE_ALLOC: &dyn VSpaceAO = &VSpaceAODummy;
static mut PAGE_ALLOC: &dyn VSpaceAO = &VSpaceAODummy;

/// Page table entry structure
///
/// Just define here. Should implement functions in specific architectures.
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug)]
pub(crate) struct PTE(pub usize);

bitflags::bitflags! {
    /// Mapping flags for page table.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MappingFlags: u64 {
        /// Persent
        const P = bit!(0);
        /// User Accessable Flag
        const U = bit!(1);
        /// Readable Flag
        const R = bit!(2);
        /// Writeable Flag
        const W = bit!(3);
        /// Executeable Flag
        const X = bit!(4);
        /// Accessed Flag
        const A = bit!(5);
        /// Dirty Flag, indicating that the page was written
        const D = bit!(6);
        /// Global Flag
        const G = bit!(7);
        /// Device Flag, indicating that the page was used for device memory
        const Device = bit!(8);
        /// Cache Flag, indicating that the page will be cached
        const Cache = bit!(9);

        /// Read | Write | Executeable Flags
        const RWX = Self::R.bits() | Self::W.bits() | Self::X.bits();
        /// User | Read | Write Flags
        const URW = Self::U.bits() | Self::R.bits() | Self::W.bits();
        /// User | Read | Executeable Flags
        const URX = Self::U.bits() | Self::R.bits() | Self::X.bits();
        /// User | Read | Write | Executeable Flags
        const URWX = Self::URW.bits() | Self::X.bits();
    }
}

/// Virtual Space Abstract Operation.
pub trait VSpaceAO: Sync {
    /// Allocate a physical page
    fn alloc_page(&self) -> PhysAddr;
    /// Free a physical page
    fn free_page(&self, paddr: PhysAddr);
}

/// A Dummy Implementation for VSpaceAO
pub struct VSpaceAODummy;

impl VSpaceAO for VSpaceAODummy {
    fn alloc_page(&self) -> PhysAddr {
        unreachable!("Dummy Alloc Page")
    }

    fn free_page(&self, _paddr: PhysAddr) {
        unreachable!("Dummy Free Page")
    }
}

#[inline]
#[allow(static_mut_refs)]
fn alloc_page() -> PhysAddr {
    unsafe { PAGE_ALLOC.alloc_page() }
}

#[inline]
#[allow(static_mut_refs)]
fn free_page(paddr: PhysAddr) {
    unsafe { PAGE_ALLOC.free_page(paddr) }
}

/// Virtual Address Space
///
/// This is just the page table defination.
/// The implementation of the page table in the specific architecture mod.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VSpace(pub(crate) PhysAddr);

impl VSpace {
    const _CHECK: () = assert!(Self::PAGE_LEVEL >= 3, "Just level >= 3 supported currently");

    /// Get the page table list through the physical address
    #[inline]
    pub(crate) const fn get_pte_list(paddr: PhysAddr) -> &'static mut [PTE] {
        paddr
            .mapped_vaddr()
            .slice_mut_with_len::<PTE>(Self::PTE_NUM_IN_PAGE)
    }

    /// Mapping a page to specific virtual page (user space address).
    ///
    /// Ensure that PageTable is which you want to map.
    /// vpn: Virtual page will be mapped.
    /// ppn: Physical page.
    /// flags: Mapping flags, include Read, Write, Execute and so on.
    pub fn map_page(
        &self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        flags: MappingFlags,
        size: MappingSize,
    ) {
        let pte_value = PTE::new_page(paddr, flags.into(), size);

        let mut pte_list = Self::get_pte_list(self.0);
        if Self::PAGE_LEVEL == 4 {
            let pte = &mut pte_list[pg_index(vaddr, 3)];
            if !pte.is_valid() {
                *pte = PTE::new_table(alloc_page());
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 3
        {
            let pte = &mut pte_list[pg_index(vaddr, 2)];
            if !pte.is_valid() && size != MappingSize::Page1GB {
                *pte = PTE::new_table(alloc_page());
            } else if size == MappingSize::Page1GB {
                // TODO: Mapping huge page 1GB
                *pte = pte_value;
                return;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 2
        {
            let pte = &mut pte_list[pg_index(vaddr, 1)];
            if !pte.is_valid() && size != MappingSize::Page2MB {
                *pte = PTE::new_table(alloc_page());
            } else if size == MappingSize::Page2MB {
                // TODO: Mapping huge page 2MB
                *pte = pte_value;
                return;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 1, map page
        pte_list[pg_index(vaddr, 0)] = pte_value;
        TLB::flush_vaddr(vaddr);
    }

    /// Unmap a page from specific virtual page (user space address).
    ///
    /// Ensure the virtual page is exists.
    /// vpn: Virtual address.
    ///
    /// TODO: Unmap huge page
    pub fn unmap_page(&self, vaddr: VirtAddr, _size: MappingSize) {
        let mut pte_list = Self::get_pte_list(self.0);
        if Self::PAGE_LEVEL == 4 {
            let pte = &mut pte_list[pg_index(vaddr, 3)];
            if !pte.is_table() {
                return;
            };
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 3
        {
            let pte = &mut pte_list[pg_index(vaddr, 2)];
            if !pte.is_table() {
                return;
            };
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 2
        {
            let pte = &mut pte_list[pg_index(vaddr, 1)];
            if !pte.is_table() {
                return;
            };
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 1, map page
        pte_list[pg_index(vaddr, 0)] = PTE(0);
        TLB::flush_vaddr(vaddr);
    }

    /// Translate a virtual adress to a physical address and mapping flags.
    ///
    /// Return None if the vaddr isn't mapped.
    /// vpn: The virtual address will be translated.
    pub fn translate(&self, vaddr: VirtAddr) -> Option<(PhysAddr, MappingFlags)> {
        let mut pte_list = Self::get_pte_list(self.0);
        if Self::PAGE_LEVEL == 4 {
            let pte = &mut pte_list[pg_index(vaddr, 3)];
            if !pte.is_table() {
                return None;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 3
        {
            let pte = &mut pte_list[pg_index(vaddr, 2)];
            if !pte.is_table() {
                return None;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 2
        {
            let pte = &mut pte_list[pg_index(vaddr, 1)];
            if !pte.is_table() {
                return None;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 1, map page
        let pte = pte_list[pg_index(vaddr, 0)];
        Some((
            PhysAddr::new(pte.paddr().raw() + pg_offest(vaddr, 0)),
            pte.flags().into(),
        ))
    }

    /// Release the page table entry.
    ///
    /// The page table entry in the user space address will be released.
    /// [Page Table Wikipedia](https://en.wikipedia.org/wiki/Page_table).
    /// You don't need to care about this if you just want to use.
    pub fn release(&self) {
        let drop_l2 = |pte_list: &[PTE]| {
            pte_list.iter().for_each(|x| {
                if x.is_table() {
                    free_page(x.paddr());
                }
            });
        };
        let drop_l3 = |pte_list: &[PTE]| {
            pte_list.iter().for_each(|x| {
                if x.is_table() {
                    drop_l2(Self::get_pte_list(x.paddr()));
                    free_page(x.paddr());
                }
            });
        };
        let drop_l4 = |pte_list: &[PTE]| {
            pte_list.iter().for_each(|x| {
                if x.is_table() {
                    drop_l3(Self::get_pte_list(x.paddr()));
                    free_page(x.paddr());
                }
            });
        };

        // Drop all sub page table entry and clear root page.
        let pte_list = &mut Self::get_pte_list(self.0)[..Self::GLOBAL_ROOT_PTE_RANGE];
        match Self::PAGE_LEVEL {
            4 => drop_l4(pte_list),
            _ => drop_l3(pte_list),
        }
        pte_list.fill(PTE(0));
    }
}

/// TLB Operation set.
/// Such as flush_vaddr, flush_all.
/// Just use it in the fn.
///
/// there are some methods in the TLB implementation
///
/// ### Flush the tlb entry through the specific virtual address
///
/// ```rust
/// TLB::flush_vaddr(arg0);  arg0 should be VirtAddr
/// ```
/// ### Flush all tlb entries
/// ```rust
/// TLB::flush_all();
/// ```
pub struct TLB;

/// This structure indicates size of the page that will be mapped.
///
/// TODO: Support More Page Size, 16KB or 32KB
/// Just support 4KB right now.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MappingSize {
    /// 4KB per page
    Page4KB,
    /// 2MB per page
    Page2MB,
    /// 1GB per page
    Page1GB,
}
