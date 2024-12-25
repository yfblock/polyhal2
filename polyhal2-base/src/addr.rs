use core::{
    // ffi::CStr,
    fmt::{Debug, Display},
    ops::Add,
};

use crate::consts::{KERNEL_OFFSET, PAGE_SIZE};

/// Physical Address Struct
#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub(crate) usize);
impl From<PhysPage> for PhysAddr {
    fn from(value: PhysPage) -> Self {
        Self(value.0 << 12)
    }
}

impl PhysAddr {
    /// Get the mapped virtual address
    /// This function always used for higher half kernel
    /// The memory will be mapped in the higher memory space.
    /// Virtual address = Physical address + kernel offset.
    pub const fn mapped_vaddr(&self) -> VirtAddr {
        VirtAddr(self.0 | KERNEL_OFFSET)
    }
}

/// Virtual Address struct
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub(crate) usize);

impl VirtAddr {
    /// Get the ptr for the given `VirtAddr`
    #[inline]
    pub fn get_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    /// Get the mut ptr for the given `VirtAddr`
    #[inline]
    pub fn get_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    // #[inline]
    // pub fn get_ref<T>(&self) -> &'static T {
    //     unsafe { &*(self.0 as *const T) }
    // }

    // #[inline]
    // pub fn get_mut_ref<T>(&self) -> &'static mut T {
    //     unsafe { &mut *(self.0 as *mut T) }
    // }

    /// Get a slice for the given `VirtAddr`.
    #[inline]
    pub fn slice_with_len<T>(&self, len: usize) -> &'static [T] {
        unsafe { core::slice::from_raw_parts(self.get_ptr(), len) }
    }

    /// Get the mut slice for the given `VirtAddr`
    #[inline]
    pub fn slice_mut_with_len<T>(&self, len: usize) -> &'static mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.get_mut_ptr(), len) }
    }

    // #[inline]
    // pub fn slice_until<T>(&self, is_valid: fn(T) -> bool) -> &'static mut [T] {
    //     let ptr = self.raw() as *mut T;
    //     unsafe {
    //         let mut len = 0;
    //         if !ptr.is_null() {
    //             loop {
    //                 if !is_valid(ptr.add(len).read()) {
    //                     break;
    //                 }
    //                 len += 1;
    //             }
    //         }
    //         core::slice::from_raw_parts_mut(ptr, len)
    //     }
    // }

    // #[inline]
    // pub fn get_cstr(&self) -> &CStr {
    //     unsafe { CStr::from_ptr(self.get_ptr::<i8>()) }
    // }
}

/// Physical Page Struct
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPage(pub(crate) usize);

impl From<PhysAddr> for PhysPage {
    fn from(value: PhysAddr) -> Self {
        Self(value.0 >> 12)
    }
}

impl Add<PhysPage> for PhysPage {
    type Output = PhysPage;

    fn add(self, rhs: PhysPage) -> Self::Output {
        PhysPage(self.0 + rhs.0)
    }
}

impl Add<usize> for PhysPage {
    type Output = PhysPage;

    fn add(self, rhs: usize) -> Self::Output {
        PhysPage(self.0 + rhs)
    }
}

/// Virtual Page
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPage(pub(crate) usize);
impl From<VirtAddr> for VirtPage {
    fn from(value: VirtAddr) -> Self {
        Self(value.0 >> 12)
    }
}

impl PhysPage {
    /// FIXME: Get the buffer for the page.
    #[inline]
    pub const fn get_buffer(&self) -> &'static mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut((self.0 << 12 | KERNEL_OFFSET) as *mut u8, PAGE_SIZE)
        }
    }
}

impl Add<usize> for VirtPage {
    type Output = VirtPage;

    fn add(self, rhs: usize) -> Self::Output {
        VirtPage(self.0 + rhs)
    }
}

impl From<VirtPage> for VirtAddr {
    fn from(value: VirtPage) -> Self {
        Self(value.to_addr())
    }
}

macro_rules! impl_multi {
    ($($t:ident),* {$($block:item)*}) => {
        macro_rules! methods {
            () => {
                $($block)*
            };
        }
        $(
            impl $t {
                methods!();
            }
        )*
    };
    ($trait:ident => ($($ty:ident),*) { $func:item }) => {
        $(
            impl $trait for $ty {
                $func
            }
        )*
    };
}

impl_multi!(Debug => (VirtAddr, VirtPage, PhysAddr, PhysPage) {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
});

impl_multi!(Display => (VirtAddr, VirtPage, PhysAddr, PhysPage) {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
});

impl_multi!(VirtAddr, VirtPage, PhysAddr, PhysPage {
    /// Create a new object from the specific value
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Get the raw number in this object
    pub const fn raw(&self) -> usize {
        self.0
    }
});

impl_multi!(VirtAddr, PhysAddr {
    /// align down the address with `align`
    pub const fn floor(&self, align: usize) -> Self {
        Self(self.0 / align * align)
    }

    /// align up the address with `align`
    pub const fn ceil(&self, align: usize) -> Self {
        Self((self.0 + align - 1) / align * align)
    }
});

impl_multi!(VirtPage, PhysPage {
    /// Get raw address from the page.
    pub const fn to_addr(&self) -> usize {
        self.0 << 12
    }

    /// Get the page from the address.
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr >> 12)
    }
});
