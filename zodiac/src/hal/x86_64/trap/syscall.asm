.code64

.text
.global syscall_return
syscall_return:
    swapgs
   
    push r15
    push r14
    push r13
    push r12
    push rbp
    push rbx

    push rdi
    mov gs:4, rsp
    mov rsp, rdi
    
    swapgs

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    
    pop r11
    pop r10
    pop r9
    pop r8
    pop rsi
    pop rdi
    pop rdx
    pop rcx
    pop rax
    
    add rsp, 16
    
    # Determine whether to use sysret or iret.
    # If returning to user space with a clean context,
    # the fast sysret path can be used;
    # otherwise, the slower iret path should be used.
    # Reference: <https://elixir.bootlin.com/linux/v6.0.9/source/arch/x86/entry/entry_64.S#L122>.

    cmp qword ptr [rsp], rcx      # sysret requires rcx = rip
    jne iret

    cmp qword ptr [rsp + 16], r11  # sysret requires r11 = rflags
    jne iret 

    test r11, 0x10100             # sysret requires rflags not contain RF and TF flags
    jnz iret

    jmp sysret

iret:
    iretq

sysret:
    add rsp, 24
    pop rsp

    sysretq

    # sysretq instruction do:
    # - load cs, ss
    # - load rflags <- r11
    # - load rip <- rcx
