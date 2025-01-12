use core::{
    // ffi::CStr,
    fmt::{Debug, Display},
};

use crate::consts::KERNEL_OFFSET;

/// Physical Address Struct
#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub(crate) usize);

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
    pub const fn get_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    /// Get the mut ptr for the given `VirtAddr`
    #[inline]
    pub const fn get_mut_ptr<T>(&self) -> *mut T {
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
    pub const fn slice_with_len<T>(&self, len: usize) -> &'static [T] {
        unsafe { core::slice::from_raw_parts(self.get_ptr(), len) }
    }

    /// Get the mut slice for the given `VirtAddr`
    #[inline]
    pub const fn slice_mut_with_len<T>(&self, len: usize) -> &'static mut [T] {
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

impl_multi!(Debug => (VirtAddr, PhysAddr) {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
});

impl_multi!(Display => (VirtAddr, PhysAddr) {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
});

impl_multi!(VirtAddr, PhysAddr {
    /// Create a new object from the specific value
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Get the raw number in this object
    pub const fn raw(&self) -> usize {
        self.0
    }

    /// align down the address with `align`
    pub const fn floor(&self, align: usize) -> Self {
        Self(self.0 / align * align)
    }

    /// align up the address with `align`
    pub const fn ceil(&self, align: usize) -> Self {
        Self(self.0.div_ceil(align) * align)
    }
});
