# racaOS

![Logo](Logo.bmp)

## Introduction

A system that has undergone multiple reconstructions and currently runs on the LoongArch64 platform. \
Birthday of the first version: 2023-01-28 \
Birthday of this version: 2025-11-20

The current version is a practical application of the enhanced version of the framekernel. \
The general idea is to provide a kernel that runs as fast as a macro kernel, as safe as a micro kernel, and as flexible as a micro kernel. \
The component system of Asterinas provides a compile-time extensible kernel, but it means that you have
to recompile everything when adding some new features. However, racaOS's kernel module system allows
you to dynamically load and unload kernel modules without recompiling the entire kernel.

The kernel is designed to be modular and extensible, allowing for easy integration of new features and drivers.
Everything is provided as kernel modules, including the framework. You can develop kernel modules as easy as developing a normal rust crate.

## Roadmap

- [x] Kernel Modules
- [x] Memory Management
- [ ] Multitask
- [ ] User space
- [ ] AHCI driver
- [ ] NVMe driver
- [ ] File system: FAT, ext2, ext4
- [ ] Bash and coreutils
- [ ] Network connection support
- [ ] Network drivers
- [ ] Alpine package keeper
- [ ] Better schedulers
- [ ] Run compilers
- [ ] DRM support
- [ ] Xorg and Wayland compositors
- [ ] More drivers

## Thanks
- [asterinas](https://github.com/asterinas) for inspiration from ostd and osdk.  
- [wenxuanjun](https://github.com/wenxuanjun) for page table examples.
