use crate::declare_env_var;

/// Kernel Offset is the offset between 
pub const KERNEL_OFFSET: usize = declare_env_var!("KERNEL_OFFSET", usize);
/// The size of the page.
pub const PAGE_SIZE: usize = declare_env_var!("PAGE_SIZE", usize);
