#include <stdio.h>

static int ack(int m, int n) {
    if (m == 0) {
        return n + 1;
    }
    if (n == 0) {
        return ack(m - 1, 1);
    }
    return ack(m - 1, ack(m, n - 1));
}

int main(void) {
    int total = 0;
    for (int rep = 0; rep < 15; rep++) {
        total = (total + ack(3, 6)) & 0x7FFFFFFF;
    }
    if (total == 7635) {
        printf("RESULT: 7635\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
