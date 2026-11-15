#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int x;
    int y;
} Point;

int main(void) {
    int total = 0;
    for (int i = 0; i < 50000; i++) {
        Point *p = (Point *)malloc(sizeof(Point));
        p->x = i;
        p->y = i * 2;
        total = (total + p->x + p->y) & 0x7FFFFFFF;
        free(p);
    }

    if (total == 1602441352) {
        printf("RESULT: 1602441352\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
