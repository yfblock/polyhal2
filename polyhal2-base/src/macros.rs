
/// Declare consts using env.
/// Passing environment variable when compiling.
#[macro_export]
macro_rules! declare_env_var {
    ($name:literal, $t:ident) => {{
        let v = option_env!($name).expect(concat!("Not found environment variable ", $name));
        match $t::from_str_radix(v, 16) {
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
