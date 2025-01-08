fn main() {
    println!(
        "cargo:rustc-env=BUILD_TARGET={}",
        std::env::var("CARGO_CFG_TARGET_ARCH").unwrap()
    );
    println!(
        "cargo:rustc-env=BUILD_ABI={}",
        std::env::var("CARGO_CFG_TARGET_ABI").unwrap()
    );
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
}
