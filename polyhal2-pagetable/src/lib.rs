//! Make Support For Multi PlatForm
//!
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]

use core::marker::PhantomData;

use arch::{pn_index, pn_offest};
use polyhal2_base::{
    addr::{PhysAddr, PhysPage, VirtAddr, VirtPage},
    bit,
};

/// PageTable for aarch64
#[cfg_attr(target_arch = "aarch64", path = "aarch64.rs")]
mod arch;

/// Page table entry structure
///
/// Just define here. Should implement functions in specific architectures.
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
pub trait VSpaceAO {
    /// Allocate a physical page 
    fn alloc_page() -> PhysPage;
    /// Free a physical page
    fn free_page(page: PhysPage);
}

/// A Dummy Implementation for VSpaceAO
pub struct VSpaceAODummy;

impl VSpaceAO for VSpaceAODummy {
    fn alloc_page() -> PhysPage {
        unreachable!("Dummy Alloc Page")
    }

    fn free_page(_page: PhysPage) {
        unreachable!("Dummy Free Page")
    }
}

/// Virtual Address Space
///
/// This is just the page table defination.
/// The implementation of the page table in the specific architecture mod.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VSpace<T: VSpaceAO>(pub(crate) PhysAddr, PhantomData<T>);

impl<T: VSpaceAO> VSpace<T> {
    /// Get the page table list through the physical address
    #[inline]
    pub(crate) const fn get_pte_list(paddr: PhysAddr) -> &'static mut [PTE] {
        paddr.mapped_vaddr().slice_mut_with_len::<PTE>(Self::PTE_NUM_IN_PAGE)
    }

    /// Mapping a page to specific virtual page (user space address).
    ///
    /// Ensure that PageTable is which you want to map.
    /// vpn: Virtual page will be mapped.
    /// ppn: Physical page.
    /// flags: Mapping flags, include Read, Write, Execute and so on.
    pub fn map_page(&self, vpn: VirtPage, ppn: PhysPage, flags: MappingFlags) {
        assert!(
            vpn.to_addr() <= Self::USER_VADDR_END,
            "This is not a valid address"
        );
        assert!(Self::PAGE_LEVEL >= 3, "Just level >= 3 supported currently");
        let mut pte_list = Self::get_pte_list(self.0);
        if Self::PAGE_LEVEL == 4 {
            let pte = &mut pte_list[pn_index(vpn, 3)];
            if !pte.is_valid() {
                *pte = PTE::new_table(T::alloc_page());
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 3
        {
            let pte = &mut pte_list[pn_index(vpn, 2)];
            if !pte.is_valid() {
                *pte = PTE::new_table(T::alloc_page());
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 2
        {
            let pte = &mut pte_list[pn_index(vpn, 1)];
            if !pte.is_valid() {
                *pte = PTE::new_table(T::alloc_page());
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 1, map page
        pte_list[pn_index(vpn, 0)] = PTE::new_page(ppn, flags.into());
        TLB::flush_vaddr(vpn.into());
    }

    /// Unmap a page from specific virtual page (user space address).
    ///
    /// Ensure the virtual page is exists.
    /// vpn: Virtual address.
    pub fn unmap_page(&self, vpn: VirtPage) {
        let mut pte_list = Self::get_pte_list(self.0);
        if Self::PAGE_LEVEL == 4 {
            let pte = &mut pte_list[pn_index(vpn, 3)];
            if !pte.is_table() {
                return;
            };
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 3
        {
            let pte = &mut pte_list[pn_index(vpn, 2)];
            if !pte.is_table() {
                return;
            };
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 2
        {
            let pte = &mut pte_list[pn_index(vpn, 1)];
            if !pte.is_table() {
                return;
            };
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 1, map page
        pte_list[pn_index(vpn, 0)] = PTE(0);
        TLB::flush_vaddr(vpn.into());
    }

    /// Translate a virtual adress to a physical address and mapping flags.
    ///
    /// Return None if the vaddr isn't mapped.
    /// vpn: The virtual address will be translated.
    pub fn translate(&self, vaddr: VirtAddr) -> Option<(PhysAddr, MappingFlags)> {
        let vpn: VirtPage = vaddr.into();
        let mut pte_list = Self::get_pte_list(self.0);
        if Self::PAGE_LEVEL == 4 {
            let pte = &mut pte_list[pn_index(vpn, 3)];
            if !pte.is_table() {
                return None;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 3
        {
            let pte = &mut pte_list[pn_index(vpn, 2)];
            if !pte.is_table() {
                return None;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 2
        {
            let pte = &mut pte_list[pn_index(vpn, 1)];
            if !pte.is_table() {
                return None;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        // level 1, map page
        let pte = pte_list[pn_index(vpn, 0)];
        Some((
            PhysAddr::new(pte.paddr().raw() + pn_offest(vaddr, 0)),
            pte.flags().into(),
        ))
    }

    /// Map the huge page page, vaddr -> vaddr
    /// 
    /// Support 1GB, 2MB
    pub unsafe fn map_huge(&self, vaddr: VirtAddr, paddr: PhysAddr, flags: MappingFlags) {
        let vpn: VirtPage = vaddr.into();
        let mut pte_list = Self::get_pte_list(self.0);
        if Self::PAGE_LEVEL == 4 {
            let pte = &mut pte_list[pn_index(vpn, 3)];
            if !pte.is_table() {
                return;
            }
            pte_list = Self::get_pte_list(pte.paddr());
        }
        pte_list[pn_index(vpn, 2)] = PTE::new_page(paddr.into(), flags.into());
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
                    T::free_page(x.paddr().into());
                }
            });
        };
        let drop_l3 = |pte_list: &[PTE]| {
            pte_list.iter().for_each(|x| {
                if x.is_table() {
                    drop_l2(Self::get_pte_list(x.paddr()));
                    T::free_page(x.paddr().into());
                }
            });
        };
        let drop_l4 = |pte_list: &[PTE]| {
            pte_list.iter().for_each(|x| {
                if x.is_table() {
                    drop_l3(Self::get_pte_list(x.paddr()));
                    T::free_page(x.paddr().into());
                }
            });
        };

        // Drop all sub page table entry and clear root page.
        let pte_list = &mut Self::get_pte_list(self.0)[..Self::GLOBAL_ROOT_PTE_RANGE];
        match Self::PAGE_LEVEL {
            4 => drop_l4(&pte_list),
            _ => drop_l3(&pte_list),
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
#[derive(Debug)]
pub enum MappingSize {
    /// 4KB per page
    Page4KB,
    // Page2MB,
    // Page1GB,
}
