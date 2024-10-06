use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Tell cargo to pass the linker script to the linker..
    println!(
        "cargo:rustc-link-arg=-T{}/../link.ld",
        manifest_dir.display()
    );

    // ..and to re-run if it changes.
    println!(
        "cargo:rerun-if-changed={}/../link.ld",
        manifest_dir.display()
    );
}
