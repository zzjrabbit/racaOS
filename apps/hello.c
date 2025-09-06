#include <stdio.h>

int main() {
    FILE *file = fopen("/input.txt", "r");
    if (file == NULL) {
        perror("Error opening file");
        return 1;
    }
    char buffer[32] = {0};
    int len = 32;
    for (int i = 0; i < 32; i++) {
        char ch = fgetc(file);
        if (ch == EOF) {
            len = i + 1;
            break;
        }
        buffer[i] = ch;
    }
    
    for (int i = 0; i < len; i++) {
        printf("%c", buffer[i]);
    }
    
    fclose(file);
    return 0;
}
