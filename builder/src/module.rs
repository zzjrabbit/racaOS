use std::{
    fs::{File, create_dir_all},
    path::Path,
    process::Command,
};

use anyhow::Result;
use zstd::stream::copy_encode;

pub fn build_modules() -> Result<Vec<String>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let module_dir = manifest_dir.join("../modules");
    let target_dir = manifest_dir.join("../target");
    let compressed_module_dir = target_dir.join("modules");

    create_dir_all(&compressed_module_dir)?;

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

        let module_path = target_dir
            .join("loongarch64-unknown-linux-musl")
            .join("release")
            .join(format!("lib{}.so", module));
        let mut module_file = File::open(module_path)?;

        let mut target_file =
            File::create(compressed_module_dir.clone().join(format!("{}.km", module)))?;

        copy_encode(&mut module_file, &mut target_file, 7)?;
    }

    Ok(modules)
}
