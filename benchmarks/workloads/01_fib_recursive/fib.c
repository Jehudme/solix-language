#include <stdio.h>

static int fib(int n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

int main(void) {
    int result = fib(28);
    if (result == 317811) {
        printf("RESULT: 317811\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
