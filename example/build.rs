use std::{env, fs};

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("Can't find available architecture");
    let base_addr: usize = match target_arch.as_str() {
        "loongarch64" => 0x90000000,
        // "aarch64" => 0x40080000,
        "aarch64" => 0xffffff8040080000,
        _ => panic!("Unsupported architecture"),
    };

    // Write a proper linker file to the target directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let content = fs::read_to_string("linker.ld.in").expect("Not found linker.ld.in");
    let replaced = content.replace("@BASE_ADDR@", &base_addr.to_string());
    let linker_path = format!("{}/linker-{}.ld", out_dir, target_arch);
    fs::write(&linker_path, replaced).expect("Failed to write linker.ld");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=linker.ld.in");
    println!("cargo::rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo::rustc-link-arg=-T{}", linker_path);
    // panic!("{:?}", linker_path)
}
