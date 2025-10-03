use cargo_metadata::MetadataCommand;

fn main() {
    let metadata = MetadataCommand::new()
        .manifest_path("../../../Cargo.toml")
        .exec()
        .expect("Failed to execute cargo metadat!");

    let workspace_members: Vec<_> = metadata
        .workspace_packages()
        .iter()
        .map(|p| p.name.clone())
        .collect();

    println!(
        "cargo:rustc-env=WORKSPACE_MEMBERS={}",
        workspace_members.join(",")
    );
}
