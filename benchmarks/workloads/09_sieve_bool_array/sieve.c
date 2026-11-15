#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

int main(void) {
    int limit = 100000;
    bool *is_prime = (bool *)malloc((limit + 1) * sizeof(bool));
    for (int i = 2; i <= limit; i++) {
        is_prime[i] = true;
    }

    for (int p = 2; p * p <= limit; p++) {
        if (is_prime[p]) {
            for (int m = p * p; m <= limit; m += p) {
                is_prime[m] = false;
            }
        }
    }

    int count = 0;
    for (int i = 2; i <= limit; i++) {
        if (is_prime[i]) {
            count++;
        }
    }
    free(is_prime);

    if (count == 9592) {
        printf("RESULT: 9592\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
