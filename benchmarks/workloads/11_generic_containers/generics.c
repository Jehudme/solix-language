#include <stdio.h>

typedef struct {
    int value;
} BoxInt;

static inline int box_get_val(const BoxInt *b) {
    return b->value;
}

int main(void) {
    int total = 0;
    for (int i = 0; i < 50000; i++) {
        BoxInt b1 = { i };
        BoxInt b2 = { i * 4 + 7 };
        total = (total + box_get_val(&b1) + box_get_val(&b2)) & 0x7FFFFFFF;
    }

    if (total == 1955257704) {
        printf("RESULT: 1955257704\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
