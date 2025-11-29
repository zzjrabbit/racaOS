use std::{path::Path, process::Command};

use anyhow::Result;

pub fn build_modules() -> Result<Vec<String>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let module_dir = manifest_dir.join("../modules");

    let mut modules = Vec::<String>::new();

    for entry in walkdir::WalkDir::new(module_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file() {
            continue;
        }

        let name = entry.file_name().to_str().unwrap();
        if name.starts_with("mostd") {
            continue;
        }
        modules.push(name.into());
    }

    for module in modules.iter() {
        let mut cargo = Command::new("cargo");
        cargo.arg("build");

        cargo.args(["--target", "loongarch64-unknown-linux-musl"]);
        cargo.args(["--package", module]);
        cargo.args(["--release"]);

        cargo.spawn().unwrap().wait().unwrap();
    }

    Ok(modules)
}
