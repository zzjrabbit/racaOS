use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use argh::FromArgs;

mod image_builder;

#[derive(FromArgs)]
#[argh(description = "racaOS bootloader and kernel builder")]
struct Args {
    #[argh(switch, short = 'b')]
    #[argh(description = "boot the constructed image")]
    boot: bool,

    #[argh(switch, short = 'k')]
    #[argh(description = "use KVM acceleration")]
    kvm: bool,

    #[argh(switch, short = 'h')]
    #[argh(description = "use HAXM acceleration")]
    haxm: bool,

    #[argh(option, short = 'c')]
    #[argh(default = "2")]
    #[argh(description = "number of CPU cores")]
    cores: usize,

    #[argh(switch, short = 's')]
    #[argh(description = "redirect serial to stdio")]
    serial: bool,
}

fn main() {
    let mut cmd = Command::new("cargo");
    cmd.current_dir("modules");
    cmd.arg("build");
    cmd.arg("--package").arg("hello");
    cmd.arg("--release");
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();

    /*let vdso_path = PathBuf::from(env!("CARGO_CDYLIB_FILE_RC_VDSO_rc_vdso"));
    println!("VDSO Path: {}", vdso_path.display());
    let mut vdso_src = File::open(vdso_path).unwrap();*/

    //let user_boot_path = PathBuf::from(env!("CARGO_BIN_FILE_USER_BOOT_user_boot"));
    //println!("User Boot Path: {}", user_boot_path.display());
    //let mut user_boot_src = File::open(user_boot_path).unwrap();

    let raca_core_path = PathBuf::from(env!("CARGO_BIN_FILE_RACA_CORE_raca_core"));
    println!("RacaCore Path: {}", raca_core_path.display());
    let mut raca_core_src = File::open(raca_core_path).unwrap();

    let hello_path = PathBuf::from("target/target/release/hello");
    println!("Hello Path: {}", hello_path.display());
    let mut hello_src = File::open(hello_path).unwrap();

    let images_path = PathBuf::from("esp");

    //let mut vdso_dest = File::create(images_path.join("libraca-libos.so")).unwrap();
    //let mut user_boot_dest = File::create(images_path.join("userboot.so")).unwrap();
    let mut hello_dest = File::create(images_path.join("hello.km")).unwrap();
    let mut raca_core_dest = File::create(images_path.join("core.so")).unwrap();

    //io::copy(&mut vdso_src, &mut vdso_dest).unwrap();
    //io::copy(&mut user_boot_src, &mut user_boot_dest).unwrap();
    io::copy(&mut hello_src, &mut hello_dest).unwrap();
    io::copy(&mut raca_core_src, &mut raca_core_dest).unwrap();

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let img_path = manifest_dir.parent().unwrap().join("racaOS.img");

    let mut files = BTreeMap::new();

    for entry in walkdir::WalkDir::new("esp") {
        if let Ok(entry) = entry {
            //println!("{:#?}",entry.path());
            if entry.file_type().is_file() {
                let mut path = entry.path().to_str().unwrap().to_string();
                for _ in 0..4 {
                    // 删除前4个字符，即"esp/"
                    path.remove(0);
                }
                let path = path.replace("\\", "/");

                files.insert(path.clone(), entry.path().to_path_buf());
            }
        }
    }

    image_builder::ImageBuilder::build(files, &img_path).unwrap();

    let args: Args = argh::from_env();

    if args.boot {
        let mut cmd = Command::new("qemu-system-x86_64");
        let drive_config = format!(
            "format=raw,file={},if=none,id=boot_disk",
            &img_path.display()
        );

        cmd.arg("-device").arg("ahci,id=ahci");
        cmd.arg("-machine").arg("q35");
        cmd.arg("-m").arg("4g");
        cmd.arg("-pflash").arg("ovmf/x86_64.fd");
        cmd.arg("-drive").arg(drive_config);
        cmd.arg("-device").arg("ide-hd,drive=boot_disk,bus=ahci.0");
        cmd.arg("-smp").arg(format!("cores={}", args.cores));
        cmd.arg("-cpu").arg("qemu64,+x2apic");
        cmd.arg("-usb");
        cmd.arg("-device").arg("qemu-xhci,id=xhci");
        /*cmd.arg("-drive")
            .arg("format=raw,file=disk.img,if=none,id=disk1");
        cmd.arg("-device").arg("ide-hd,drive=disk1,bus=ahci.2");
        cmd.arg("-drive")
            .arg("format=raw,file=data.img,if=none,id=disk2");
        cmd.arg("-device").arg("nvme,drive=disk2,serial=1234");*/
        cmd.arg("-net").arg("nic");

        if args.kvm {
            cmd.arg("--enable-kvm");
        }
        if args.haxm {
            cmd.arg("-accel").arg("hax");
        }
        if args.serial {
            cmd.arg("-serial").arg("stdio");
        }

        let mut child = cmd.spawn().unwrap();
        child.wait().unwrap();
    }
}
