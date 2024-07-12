# racaOS compile environment install

## Prepare
Install rust and qemu first.

## Install
Install the nightly toolchain.
``` shell
rustup install nightly
rustup default nightly
```
Add target.
``` shell
rustup target add x86_64-unknown-none
rustup target add x86_64-unknown-uefi
```
Install some packages.
``` shell
rustup component add rust-src
```
