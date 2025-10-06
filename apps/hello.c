#include <stdio.h>

int main() {
    FILE *file = fopen("/part0/input.txt", "r");
    if (file == NULL) {
        perror("Error opening file");
        return 1;
    }
    char buffer[22 * 1024] = {0};
    int len = 22 * 1024;
    for (int i = 0; i < len; i++) {
        char ch = fgetc(file);
        if (ch == EOF) {
            len = i;
            break;
        }
        buffer[i] = ch;
    }
    
    //printf("Length: %d \n", len);
    
    for (int i = 0; i < len; i++) {
        printf("%c", buffer[i]);
    }
    
    fclose(file);
    return 0;
}
