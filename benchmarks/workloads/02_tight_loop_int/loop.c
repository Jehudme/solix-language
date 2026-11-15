#include <stdio.h>

int main(void) {
    int total = 0;
    for (int i = 0; i < 1000000; i++) {
        total = (total + (i ^ (i % 7))) & 0x7FFFFFFF;
    }

    if (total == 1783293643) {
        printf("RESULT: 1783293643\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
