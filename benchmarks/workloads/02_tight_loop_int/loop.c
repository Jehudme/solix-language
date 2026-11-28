#include <stdio.h>

int main(void) {
    int total = 0;
    for (int i = 0; i < 8000000; i++) {
        total = (total + (i ^ (i % 7))) & 0x7FFFFFFF;
    }

    if (total == 342161131) {
        printf("RESULT: 342161131\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
