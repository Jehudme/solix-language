#include <stdio.h>

#define N 60

int main(void) {
    int a[N * N];
    int b[N * N];

    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            a[i * N + j] = (i + j) % 100;
            b[i * N + j] = (i * j) % 100;
        }
    }

    int trace = 0;
    for (int i = 0; i < N; i++) {
        int c_ii = 0;
        for (int k = 0; k < N; k++) {
            c_ii = (c_ii + a[i * N + k] * b[k * N + i]) & 0x7FFFFFFF;
        }
        trace = (trace + c_ii) & 0x7FFFFFFF;
    }

    if (trace == 9118300) {
        printf("RESULT: 9118300\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
