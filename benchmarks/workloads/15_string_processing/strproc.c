#include <stdio.h>
#include <string.h>

static int process(const char *s, int factor) {
    if (strcmp(s, "alpha") == 0) return factor * 2;
    if (strcmp(s, "beta") == 0) return factor * 3;
    if (strcmp(s, "gamma") == 0) return factor * 5;
    return factor;
}

int main(void) {
    const char *words[3] = { "alpha", "beta", "gamma" };
    int total = 0;
    for (int i = 0; i < 100000; i++) {
        const char *w = words[i % 3];
        total = (total + process(w, i % 20)) & 0x7FFFFFFF;
    }

    if (total == 3166647) {
        printf("RESULT: 3166647\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
