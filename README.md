# racaOS

![Logo](Logo.bmp)

## Introduction
A system that has undergone multiple reconstructions and runs on the x86_64 platform\
Birthday:2023-01-28 \
Thanks to [phil-opp]( https://github.com/phil-opp ) for x86_64 crate. \
Thanks to the rCore team（ https://gitee.com/rcore-os ）They provided us with guidance templates, driver templates, and interrupt refactoring ideas\
Thank you [wenxuanjun]（ https://github.com/wenxuanjun ）He ushered racaOS into the UEFI era\

### Directories
raca_core:main \
esp:The system startup directory,Copy and paste all files from it onto a FAT formatted USB drive to start racaOS \
Docs:Documents \

## Documents
Please look at \
[Documents](Docs/index.md)


## TODO

- [x] 64 bits
- [x] English display
- [x] Qemu x2APIC interrupts
- [x] Multitask
- [x] Clock
- [x] Real machine x2APIC interrupts
- [x] Memory management
- [x] SMP
- [ ] AHCI driver
- [ ] File system
- [ ] Start time calculation
- [ ] Chinese display
- [ ] Perfect GUI
- [ ] Log in with passwords
- [ ] Network drivers
- [ ] Sound card drivers
- [ ] Network connection support
- [ ] Perfect API
- [ ] Minor support for C library
- [ ] GUI Library
