fn main() {
    println!("cargo:rustc-link-arg=-T{}", "example/linker.ld");
}
