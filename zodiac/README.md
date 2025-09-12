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

When you want to run user tasks, you can do something like this:

```rust
use zodiac::task::{ReturnReason, Task, TaskBuilder, UserContext};

let thread = TaskBuilder::default()
    .entry(thread_entry)
    .data((user_entry, user_stack))
    .build()?;

fn thread_entry() -> ! {
    let mut user_context = {
        let task = Task::current();
        let (entry, stack) = task.data().downcast_ref::<(usize, usize)>().unwrap();

        UserContext::new(*entry, *stack)
    };
    
    loop {
        let return_reason = user_context.excute(// Some clossure that tells if there is a kernel event.);
        
        match return_reason {
            ReturnReason::Syscall => {
                // Handle Syscall
            }
            ReturnReason::Exception(exception) => {
                // Handle Exception
            }
            ReturnReason::KernelEvent => {
                // Handle Kernel Event
            }
        }
    }
}

```

For more examples, please refer to [racaOS](https://github.com/zzjrabbit/racaOS).

## Roadmap

- [x] Bootloader: limine
- [x] Memory Management
- [x] SMP
- [x] Interrupt handling
- [x] Multitask
- [x] Syscall
- [x] PCIe
- [ ] MSI & MSI-X
- [ ] IOMMU
