#include <stdio.h>

typedef struct {
    int base;
} Closure;

static inline int adder(const Closure *c, int n) {
    return n + c->base;
}

int main(void) {
    Closure c = { 7 };
    int total = 0;
    for (int i = 0; i < 400000; i++) {
        total = (total + adder(&c, i % 100)) & 0x7FFFFFFF;
    }

    if (total == 22600000) {
        printf("RESULT: 22600000\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
