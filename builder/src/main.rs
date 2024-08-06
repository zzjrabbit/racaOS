use argh::FromArgs;
use std::process::Command;
use std::{collections::BTreeMap, fs::File, io, path::Path};

mod gz_builder;
mod image_builder;

#[derive(FromArgs)]
#[argh(description = "TrashOS bootloader and kernel builder")]
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
    let init_path = env!("CARGO_BIN_FILE_INIT_init");
    let shell_path = env!("CARGO_BIN_FILE_SHELL_shell");

    let app_path = "esp/RACA/app64/".to_string();

    let init_dest = app_path.clone() + "init.rae";
    let shell_dest = app_path.clone() + "shell.rae";

    io::copy(
        &mut File::open(Path::new(init_path)).unwrap(),
        &mut File::create(init_dest).unwrap(),
    )
    .unwrap();
    io::copy(
        &mut File::open(Path::new(shell_path)).unwrap(),
        &mut File::create(shell_dest).unwrap(),
    )
    .unwrap();

    let raca_core_path = env!("CARGO_BIN_FILE_RACA_CORE_raca_core");
    println!(
        "Building UEFI disk image for kernel at {:#?}",
        &raca_core_path
    );

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let img_path = manifest_dir.parent().unwrap().join("racaOS.img");

    let mut files = BTreeMap::new();

    gz_builder::compress_file(
        raca_core_path.to_string(),
        "esp/RACA/system64/core.sys".into(),
    );
    //io::copy(&mut File::open(Path::new(raca_core_path)).unwrap(), &mut File::create("esp/RACA/system64/core.sys").unwrap()).unwrap();
    //println!("Hello, world!");

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

        cmd.arg("-machine").arg("q35");
        cmd.arg("-m").arg("4g");
        cmd.arg("-pflash").arg("ovmf/x86_64.fd");
        cmd.arg("-drive").arg(drive_config);
        cmd.arg("-device").arg("nvme,drive=boot_disk,serial=1234");
        cmd.arg("-smp").arg(format!("cores={}", args.cores));
        cmd.arg("-cpu").arg("qemu64,+x2apic");
        cmd.arg("-device").arg("ahci,id=ahci");
        cmd.arg("-usb");
        cmd.arg("-device").arg("qemu-xhci,id=xhci");
        cmd.arg("-drive")
            .arg("format=raw,file=disk.img,if=none,id=disk1");
        cmd.arg("-device").arg("ide-hd,drive=disk1,bus=ahci.2");
        cmd.arg("-drive")
            .arg("format=raw,file=data.img,if=none,id=disk2");
        cmd.arg("-device").arg("nvme,drive=disk2,serial=1234");

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
