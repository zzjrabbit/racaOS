num = ["0","1","2","3","4","5","6","7","8","9","a","b","c","d","e","f"]

for i in num:
    for j in num:
        print(f"        idt[0x{i}{j}].set_handler_addr(VirtAddr::new(intentry{i}{j} as u64));")

