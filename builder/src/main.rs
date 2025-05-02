use argh::FromArgs;
use cpio::{NewcBuilder, newc::ModeFileType, write_cpio};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

#[derive(FromArgs)]
#[argh(description = "racaOS bootloader and kernel builder")]
struct Args {
    #[argh(switch, short = 'b')]
    #[argh(description = "boot the constructed image")]
    boot: bool,

    #[argh(switch, short = 'k')]
    #[argh(description = "use KVM acceleration")]
    kvm: bool,

    #[argh(switch, short = 'w')]
    #[argh(description = "use Hyper-V acceleration")]
    whpx: bool,

    #[argh(option, short = 'c')]
    #[argh(default = "4")]
    #[argh(description = "number of CPU cores")]
    cores: usize,

    #[argh(switch, short = 's')]
    #[argh(description = "redirect serial to stdio")]
    serial: bool,
}

fn main() {
    let img_path = build_img();
    println!("starting qemu");
    let args: Args = argh::from_env();

    if args.boot {
        let mut cmd = Command::new("qemu-system-x86_64");

        let ovmf_path = PathBuf::from("ovmf/x86_64.fd");
        let ovmf_config = format!("if=pflash,format=raw,file={}", ovmf_path.display());

        cmd.arg("-machine").arg("q35");
        cmd.arg("-drive").arg(ovmf_config);
        cmd.arg("-m").arg("512m");
        cmd.arg("-smp").arg(format!("cores={}", args.cores));
        cmd.arg("-cpu").arg("qemu64,+x2apic");

        if let Some(backend) = match std::env::consts::OS {
            "linux" => Some("pa"),
            "macos" => Some("coreaudio"),
            "windows" => Some("dsound"),
            _ => None,
        } {
            cmd.arg("-audiodev").arg(format!("{backend},id=sound"));
            cmd.arg("-machine").arg("pcspk-audiodev=sound");
            cmd.arg("-device").arg("intel-hda");
            cmd.arg("-device").arg("hda-output,audiodev=sound");
        }

        //let drive_config = format!("if=none,format=raw,id=disk1,file={}", img_path.display());
        //cmd.arg("-device").arg("ahci,id=ahci");
        //cmd.arg("-device").arg("ide-hd,drive=disk1,bus=ahci.0");
        //cmd.arg("-drive").arg(drive_config);

        let drive_config = format!("if=none,format=raw,id=disk2,file={}", img_path.display());
        cmd.arg("-device").arg("nvme,drive=disk2,serial=deadbeef");
        cmd.arg("-drive").arg(drive_config);

        if args.kvm {
            cmd.arg("--enable-kvm");
        }
        if args.whpx {
            cmd.arg("-accel").arg("whpx");
        }
        if args.serial {
            cmd.arg("-serial").arg("stdio");
        }

        let mut child = cmd.spawn().unwrap();
        child.wait().unwrap();
    }
}

fn build_img() -> PathBuf {
    let _manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut config_file = File::open("config.toml").unwrap();
    let mut config_data = String::new();
    config_file.read_to_string(&mut config_data).unwrap();
    let config = toml::Table::from_str(&config_data).unwrap();

    let _esp_config = config.get("esp").unwrap().as_table().unwrap();
    let initramfs_config = config.get("initramfs").unwrap().as_table().unwrap();

    let kernel_path = Path::new(env!("CARGO_BIN_FILE_KERNEL"));
    println!("Building UEFI disk image for kernel at {:#?}", &kernel_path);
    let mut kernel_src = File::open(kernel_path).unwrap();

    let images_path = PathBuf::from("esp");

    let mut kernel_dest = File::create(images_path.join("boot").join("kernel")).unwrap();

    std::io::copy(&mut kernel_src, &mut kernel_dest).unwrap();

    let initramfs_path = PathBuf::from("initramfs");

    for user_program in initramfs_config.get("users").unwrap().as_array().unwrap() {
        let name = user_program.as_str().unwrap();
        build_user_program(name);
    }

    for entry in walkdir::WalkDir::new("target/x86_64-unknown-none/release")
        .max_depth(1)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_str().unwrap();
            if !file_name.starts_with('.') && !file_name.ends_with(".d") {
                println!("found user program: `{file_name}`");
                let user_program_path =
                    PathBuf::from("target/x86_64-unknown-none/release/".to_string() + file_name);
                let mut user_program_src = File::open(user_program_path).unwrap();

                let mut user_program_dest =
                    File::create(initramfs_path.join("bin").join(file_name)).unwrap();

                io::copy(&mut user_program_src, &mut user_program_dest).unwrap();
            }
        }
    }

    build_initramfs();

    println!("initramfs built!");

    build_image_from_dir("esp", "racaOS.img");

    println!("image built!");

    PathBuf::from("racaOS.img")
}

fn build_user_program(name: &str) {
    let mut cmd = Command::new("cargo");
    //cmd.current_dir("apps");
    cmd.arg("build");
    cmd.arg("--package").arg(name);
    cmd.arg("--release");
    cmd.arg("--target").arg("x86_64-unknown-none");
    cmd.status().unwrap();
}

fn build_initramfs() {
    let mut initramfs_file = File::create("esp/boot/initramfs").unwrap();
    let mut inputs = Vec::new();

    for entry in walkdir::WalkDir::new("initramfs").into_iter().flatten() {
        if entry.file_type().is_file() {
            let mut path = entry.path().to_str().unwrap().to_string();
            for _ in 0..="initramfs".len() {
                path.remove(0);
            }

            #[cfg(target_os = "windows")]
            let path = path.replace("\\", "/");

            println!("{} {}", path, entry.path().to_path_buf().display());

            let path_out = entry.path().to_path_buf().clone();

            inputs.push((path, path_out));
        }
    }

    write_cpio(
        inputs.iter().map(|(path_in, path)| {
            (
                NewcBuilder::new(path_in).set_mode_file_type(ModeFileType::Regular),
                File::open(path).unwrap(),
            )
        }),
        &mut initramfs_file,
    )
    .unwrap();
}

fn build_image_from_dir(dir: &str, image_file: &str) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let img_path = manifest_dir.parent().unwrap().join(image_file);

    let mut files = BTreeMap::new();

    for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
        if entry.file_type().is_file() {
            let mut path = entry.path().to_str().unwrap().to_string();
            for _ in 0..=dir.len() {
                path.remove(0);
            }
            let path = path.replace('\\', "/");

            files.insert(path.clone(), entry.path().to_path_buf());
        }
    }

    builder::ImageBuilder::build(files, &img_path).unwrap();
}
