# Zodiac

Zodiac is a framework for framekernel, inspired by [ostd](https://github.com/asterinas/asterinas/tree/main/ostd)

## Usage

First, you need to add zodiac to your project's Cargo.toml file:

```toml
[dependencies]
zodiac = "0.1.0"
```

Then, you need a main function with the attribute `#[zodiac::main]` and a panic handler with the attribute `#[zodiac::panic_handler]`:

```rust
#[zodiac::main]
pub fn main() {
    // Your code here
}

#[zodiac::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}", info);
    loop {
        core::hint::spin_loop();
    }
}
```

And there are other necessary options, which are listed below:

- allocator
- scheduler

If you like, you can also specify the logger.

For more examples, please refer to [racaOS](https://github.com/zzjrabbit/racaOS).

## Roadmap

- [x] Bootloader: limine
- [x] Memory Management
- [x] SMP
- [x] Interrupt handling
- [x] Multitask
- [x] Syscall
- [ ] PCIe
- [ ] MSI & MSI-X
- [ ] IOMMU
