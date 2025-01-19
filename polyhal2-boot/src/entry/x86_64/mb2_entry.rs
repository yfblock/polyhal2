//! Multiboot Header And Defination
//!
//! <https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html>
//!
//! The layout of the Multiboot2 header must be as follows:
//! Offset	Type	Field Name	Note
//! 0	u32	magic	required
//! 4	u32	architecture	required
//! 8	u32	header_length	required
//! 12	u32	checksum	required
//! 16-XX		tags	required
pub const MAGIC: u32 = 0xE85250D6;
/// Spec: "means 32-bit (protected) mode of i386".
/// Caution: This is confusing. If you use the EFI64-tag
/// on an UEFI system, the machine will boot into `64-bit long mode`.
/// Therefore this tag should be understood as "arch=x86|x86_64".
// pub const ARCH: u32 = 