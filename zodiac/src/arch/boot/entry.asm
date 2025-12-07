.extern BOOT_DATA
.extern ap_rust_entry

.align 16
ap_asm_entry:
    bl ap_asm_entry

    li.w $r12, 0x11
    csrwr $r12, 0x180

    li.d $r12, 0
    li.d $r13, 1 << 8
    csrxchg	$r12, $r13, 0x80
    
    li.w $r12, 0xb0
    csrwr $r12, 0x00
    li.w $r12, 0x04
    csrwr $r12, 0x01
    li.w $r12, 0x00
    csrwr $r12, 0x02
    
    la.pcrel $r12, BOOT_DATA
    ld.d $r3, $r12, 0
    ld.d $r2, $r12, 8
    
    bl ap_rust_entry
