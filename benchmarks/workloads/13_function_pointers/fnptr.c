#include <stdio.h>

static int fn1(int x) { return x * 2 + 1; }
static int fn2(int x) { return x * 3 - 2; }
static int fn3(int x) { return x * 5 + 4; }

int main(void) {
    int total = 0;
    for (int i = 0; i < 2000000; i++) {
        int idx = i % 3;
        int (*f)(int) = (idx == 0) ? fn1 : ((idx == 1) ? fn2 : fn3);
        total = (total + f(i % 50)) & 0x7FFFFFFF;
    }

    if (total == 165333298) {
        printf("RESULT: 165333298\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
