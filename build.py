import os
import sys
import argparse
import time

rust_dirs = ["device"]
rust_output = ["device"]
rust_copy_to = ["RACA/system64/core.sys"]
rust_target_file = ["none"]
asm_files = []
apps = ["hello1" ,"hello2"]
in_test = [False,False]

def run_command(command):
    code = os.system(command)
    if code != 0:
        sys.exit(code)

def main():
    parser = argparse.ArgumentParser(description="raca build script.")
    parser.add_argument('-m','--mode',type=str,help="output mode",default="release",choices=["debug","release"])
    parser.add_argument('-r','--run',action='store_true')
    parser.add_argument('-i','--install',type=str,help="install path",default="")
    parser.add_argument("-b", "--build", help="build the project", action="store_true")
    parser.add_argument("-c","--count",help="Count the number of codes.",action="store_true")
    parser.add_argument("-f","--format",help="Format the codes.",action="store_true")
    parser.add_argument("-t","--to_vmware",help="Build for VMware.",action="store_true")
    args = parser.parse_args()
    args = parser.parse_args()
    com_mode = "" if args.mode == "debug" else "release"

    if args.format:
        os.system("cargo fmt")
    if args.count:
        os.system("cloc ./ --exclude-dir=target,.VSCodeCounter,raca_loader")
    if args.build:

        start_time = time.perf_counter_ns()

        for asm_file in asm_files:
            run_command(f"nasm asm/{asm_file}.asm -o raca_core/src/{asm_file}.bin")

        # run_command(f"cd apps;cargo build --{com_mode}")
        for (idx,app) in enumerate(apps):
            if not in_test[idx]:
                run_command(f"cd apps;cargo build --package {app} --{com_mode}")
                run_command(f"cp apps/target/x86_64-unknown-none/{args.mode}/{app} apps/{app}.rae")
            else:
                run_command(f"cd apps/test;rustc {app}.rs --target x86_64-unknown-none")
                run_command(f"cp apps/test/{app} apps/{app}.rae")

        print("APP compiled.")

        for (d,output,copy_to,target) in zip(rust_dirs,rust_output,rust_copy_to,rust_target_file):
            if target.endswith(".json"):
                if args.mode == "release":
                    run_command(f"cargo build --target {target} --package {d} --{com_mode}")
                else:
                    run_command(f"cargo build --target {target} --package {d}")
                name = target.split(".json")[0]
                run_command(f"cp target/{name}/{args.mode}/{output} esp/{copy_to}")
            else:
                if args.mode == "release":
                    run_command(f"cargo build --package {d} --{com_mode}")
                else:
                    run_command(f"cargo build --package {d}")
                run_command(f"cp target/x86_64-unknown-{target}/{args.mode}/{output} esp/{copy_to}")

        end_time = time.perf_counter_ns()

        print(f"构建时间:{(end_time-start_time)/1000000000:.2f}s")

    time.sleep(1)

    if args.to_vmware:
        run_command("RACA_SYSTEM_LOOP=$(sudo losetup -P -f --show system.img);sudo mount ${RACA_SYSTEM_LOOP}p1 /mnt/raca_system --mkdir;sudo cp -rf esp/system/* /mnt/raca_system;sudo cp -rf esp/boot/* /mnt/raca_system;sudo umount /mnt/raca_system;sudo losetup -d $RACA_SYSTEM_LOOP")

        #run_command(f"sudo mount system.img /mnt/raca_system --mkdir")
        #run_command(f"sudo cp -rf esp/* /mnt/raca_system")
        #run_command(f"sudo umount /mnt/raca_system")
        run_command(f"qemu-img convert -f raw -O vmdk system.img ./vmware/racaOS/racaOS-0.vmdk")

    if args.install != "":
        path = args.install
        run_command(f"cp -r esp/* {path}")

    if args.run:

        x2apic = "-cpu qemu64,+x2apic"
        mem = "-m 4g"
        smp = "-smp 4,cores=4"

        ahci = "-device ahci,id=ahci"

        uefi = "-drive file=ovmf/x86_64.fd,format=raw,if=pflash"
        boot_disk = "-drive file=fat:rw:esp,format=raw"
        disk1 = "-drive file=disk.img,id=disk1,if=none,format=raw -device ide-hd,drive=disk1,bus=ahci.2"
        disk2 = "-drive file=data.img,id=disk2,if=none,format=raw -device nvme,drive=disk2,serial=1234"

        serial = "-serial stdio"

        kvm = "-enable-kvm"

        net = "-net none"

        run_command(f"qemu-system-x86_64 -d int,cpu_reset -D qemu.log {uefi} {boot_disk} {ahci} {disk1} {disk2} {serial} {x2apic} {mem} {smp} {kvm} {net}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
