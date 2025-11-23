fn main() {
    println!("cargo:rustc-link-arg=-T./init/linker.ld");
    println!("cargo:rerun-if-changed=linker.ld");
}
