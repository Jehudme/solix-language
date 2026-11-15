#include <stdio.h>

static int compute(int x, int y) {
    return x * 3 + y * 7;
}

int main(void) {
    int total = 0;
    for (int i = 0; i < 50000; i++) {
        total = (total + compute(i % 100, i % 50)) & 0x7FFFFFFF;
    }

    if (total == 16000000) {
        printf("RESULT: 16000000\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
