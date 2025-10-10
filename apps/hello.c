#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <sys/syscall.h>
#include <sys/types.h>

int main() {
    chdir("./part0");
    
    if (fork() == 0) {
        printf("Child process\n");
        return 0;
    }
    
    for (int i = 0; i < 100000000; i++) {
        asm volatile("nop");
    }
    
    printf("Parent process\n");
    return 0;
}
