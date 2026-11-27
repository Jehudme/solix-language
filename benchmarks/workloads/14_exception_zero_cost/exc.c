#include <stdio.h>
#include <setjmp.h>

static jmp_buf g_buf;

int main(void) {
    int caught_count = 0;
    int normal_sum = 0;

    for (int i = 0; i < 250000; i++) {
        normal_sum = (normal_sum + (i % 10)) & 0x7FFFFFFF;
    }

    for (int i = 0; i < 2500; i++) {
        int val = setjmp(g_buf);
        if (val == 0) {
            longjmp(g_buf, i + 1);
        } else {
            caught_count += (val - 1);
        }
    }

    int checksum = (normal_sum + caught_count) & 0x7FFFFFFF;
    if (checksum == 4248750) {
        printf("RESULT: 4248750\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
