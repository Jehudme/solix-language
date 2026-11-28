#include <stdio.h>

typedef struct {
    int value;
} BoxInt;

static inline int box_get_val(const BoxInt *b) {
    return b->value;
}

int main(void) {
    int total = 0;
    for (int i = 0; i < 800000; i++) {
        BoxInt b1 = { i };
        BoxInt b2 = { i * 4 + 7 };
        total = (total + box_get_val(&b1) + box_get_val(&b2)) & 0x7FFFFFFF;
    }

    if (total == 128282240) {
        printf("RESULT: 128282240\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
