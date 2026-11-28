#include <stdio.h>

static int fib(int n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

int main(void) {
    int result = fib(32);
    if (result == 2178309) {
        printf("RESULT: 2178309\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
