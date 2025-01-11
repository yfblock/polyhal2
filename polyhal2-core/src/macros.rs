
/// Declare consts using env.
/// Passing environment variable when compiling.
#[macro_export]
macro_rules! declare_env_var {
    ($name:literal, $t:ident) => {{
        match $t::from_str_radix(env!($name), 16) {
            Ok(d) => d,
            _ => 0,
        }
    }};
}

/// Get a number for a specific index
#[macro_export]
macro_rules! bit {
    ($index:literal) => {
        (1 << $index)
    };
}
