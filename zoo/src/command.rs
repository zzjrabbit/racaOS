use crate::{
    arch::get_default_arch,
    cargo::get_target_directory,
    cli::{BuildArgs, CommonArgs, RunArgs},
    config::Config,
};

use anyhow::{Context, Result};
use fatfs::{FileSystem, FormatVolumeOptions, FsOptions, format_volume};
use gpt::GptConfig;
use gpt::disk::LogicalBlockSize;
use gpt::mbr::ProtectiveMBR;
use gpt::partition_types::EFI;
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, io::Write, process::Command};
use std::{fs, io};
use std::{io::Seek, io::SeekFrom};
use tempfile::NamedTempFile;

type Files = BTreeMap<&'static str, PathBuf>;

pub fn build(config: &Config, args: &BuildArgs) {
    build_kernel(config, &args.common_args);
}

pub fn run(config: &Config, args: &RunArgs) {
    let image_path = build_kernel(config, &args.common_args);
    let target = args.common_args.target_arch.unwrap_or(get_default_arch());

    let mut qemu = Command::new(target.system_qemu());
    if let Some(qemu_conf) = &config.qemu {
        qemu.args(&qemu_conf.args);
    }

    if let Some(backend) = match std::env::consts::OS {
        "linux" => Some("pa"),
        "macos" => Some("coreaudio"),
        "windows" => Some("dsound"),
        _ => None,
    } {
        qemu.arg("-audiodev").arg(format!("{backend},id=sound"));
        qemu.arg("-machine").arg("pcspk-audiodev=sound");
        qemu.arg("-device").arg("intel-hda");
        qemu.arg("-device").arg("hda-output,audiodev=sound");
    }

    qemu.arg("-device").arg("nvme,drive=disk,serial=deadbeef");
    let param = "if=none,format=raw,id=disk";
    qemu.args(["-drive", &format!("{param},file={}", image_path.display())]);

    let param = "if=pflash,format=raw";
    let ovmf_path = Prebuilt::fetch(Source::LATEST, "target/ovmf")
        .expect("failed to update prebuilt")
        .get_file(match target {
            crate::arch::Arch::X86_64 => Arch::X64,
        }, FileType::Code);
    qemu.args(["-drive", &format!("{param},file={}", ovmf_path.display())]);
    
    if let Some(qemu_config) = &config.qemu {
        if let Some(true) = qemu_config.hw_virt {
            if let Some(opt) = match std::env::consts::OS {
                "linux" => Some("-enable-kvm"),
                _ => None,
            } {
                qemu.arg(opt);
            }
        }
        
        if let Some(serial_target) = &qemu_config.serial_target {
            qemu.arg("-serial").arg(serial_target);
        }
        
        if let Some(smp_cores) = &qemu_config.smp_cores {
            qemu.arg("-smp").arg(format!("{}", smp_cores));
        }
        
        if let Some(memory_size) = &qemu_config.memory_size {
            qemu.arg("-m").arg(memory_size);
        }
    }

    qemu.spawn().unwrap().wait().unwrap();
}

pub fn build_kernel(config: &Config, args: &CommonArgs) -> PathBuf {
    let build_conf = &config.build;

    let target_dir = get_target_directory();
    let kernel_crate = build_conf.kernel_crate.clone();

    let image_path = target_dir.join(format!("{}.img", kernel_crate));

    let mut cargo = Command::new("cargo");
    cargo.arg("build");

    let profile = if let Some(profile) = args.build_args.profile() {
        profile
    } else {
        build_conf.profile.clone().unwrap_or("dev".into())
    };
    cargo.arg(&format!("--profile={}", profile));

    let features = build_conf.features.join(" ") + args.build_args.features.join(" ").as_str();
    cargo.args(["--features", &features]);

    let target = args.target_arch.unwrap_or(get_default_arch());
    cargo.args(["--target", target.triple()]);
    
    cargo.args(["--package", &kernel_crate]);
    
    cargo.spawn().unwrap().wait().unwrap();

    let kernel_elf_path = target_dir
        .join(target.triple())
        .join(match profile.as_str() {
            "dev" => "debug",
            _ => "release",
        })
        .join(&kernel_crate);
    
    let mut limine_conf_file = NamedTempFile::new().unwrap();

    let limine_conf = include_str!("../assets/limine.conf");
    let limine_conf = limine_conf.replace("#OS_NAME", &kernel_crate);
    limine_conf_file.write(limine_conf.as_bytes()).unwrap();

    let limine_conf_path = limine_conf_file.path().to_path_buf();
    
    limine_conf_file.flush().unwrap();
    
    let mut limine_file = NamedTempFile::new().unwrap();
    let limine = include_bytes!("../assets/BOOTX64.EFI");
    limine_file.write(limine).unwrap();
    
    let limine_path = limine_file.path().to_path_buf();
    
    limine_file.flush().unwrap();
    
    let mut files = BTreeMap::new();
    files.insert("kernel", kernel_elf_path);
    files.insert("efi/boot/bootx64.efi", limine_path);
    files.insert("limine.conf", limine_conf_path);
    
    build_img(files, &image_path).unwrap();

    image_path
}

fn build_img(files: Files, image_path: &Path) -> Result<()> {
    let fat_partition = NamedTempFile::new()?;
    create_fat(&files, fat_partition.path())?;
    create_disk(fat_partition.path(), image_path)?;
    fat_partition.close()?;
    Ok(())
}

fn create_fat(files: &Files, out_path: &Path) -> Result<()> {
    let fat_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)?;

    const ADDITIONAL_SPACE: u64 = 1024 * 96;
    let total_size: u64 = files
        .values()
        .map(|p| fs::metadata(p).map(|m| m.len()))
        .sum::<Result<u64, _>>()
        .context("Failed to read files metadata")?
        + ADDITIONAL_SPACE;
    fat_file.set_len(total_size)?;

    format_volume(&fat_file, FormatVolumeOptions::new())?;
    let filesystem = FileSystem::new(&fat_file, FsOptions::new())
        .context("Failed to open FAT file system of UEFI FAT file")?;

    for (target_path, source) in files {
        let path = Path::new(&target_path);
        let root_dir = filesystem.root_dir();
        let ancestors = path.ancestors().collect::<Vec<_>>();

        for ancestor in ancestors.iter().skip(1).rev().skip(1) {
            root_dir.create_dir(&ancestor.to_string_lossy())?;
        }

        let mut new_file = root_dir.create_file(target_path)?;
        new_file.truncate()?;
        io::copy(&mut fs::File::open(source)?, &mut new_file)?;
    }
    Ok(())
}

fn create_disk(fat_image: &Path, out_path: &Path) -> Result<()> {
    let mut disk = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)?;

    let partition_size = fs::metadata(fat_image)?.len();
    let disk_size = partition_size + 1024 * 64;
    disk.set_len(disk_size)?;

    let mbr = ProtectiveMBR::with_lb_size((disk_size / 512) as u32);
    mbr.overwrite_lba0(&mut disk)?;

    let block_size = LogicalBlockSize::Lb512;
    let mut gpt = GptConfig::new()
        .writable(true)
        .logical_block_size(block_size)
        .create_from_device(Box::new(&mut disk), None)
        .context("Failed to create GPT structure in file")?;
    gpt.update_partitions(Default::default())?;

    let part_id = gpt.add_partition("boot", partition_size, EFI, 0, None)?;
    let start_offset = gpt
        .partitions()
        .get(&part_id)
        .context("Failed to open boot partition after creation")?
        .bytes_start(block_size)?;

    gpt.write()?;
    disk.seek(SeekFrom::Start(start_offset))?;
    io::copy(&mut File::open(fat_image)?, &mut disk)?;

    Ok(())
}
