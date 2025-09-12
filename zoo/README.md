# Zoo

Zoo is a OS development tool. It automatically generates a virtual disk image and runs qemu.
This project is inspired by [osdk](https://github.com/asterinas/asterinas/tree/main/osdk)

## Configuration

Configurations should be written in zoo.toml.

Example:
``` toml
[build]
kernel_crate = "kernel"
profile = "dev"
features = []

[qemu]
args = []
hw_virt = true # enables hardware virtualization
serial_target = "stdio"
smp_cores = 2 # number of cores to use
memory_size = 1m
```

## Subcommands

### build
Builds the virtual disk image.

- --profile <profile_name>
Sets the profile to use.
- --release
Builds the kernel in release mode.
- --features <feature_name>
Builds the kernel with the specified feature.
- --no-default-features
Builds the kernel without default features.
- --config KEY=VALUE
Override the configurations in zoo.toml.

### run
Runs the virtual machine.

This command shares most of the options with the build command.
