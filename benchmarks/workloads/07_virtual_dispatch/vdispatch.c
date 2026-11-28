#include <stdio.h>

typedef struct Shape Shape;
struct Shape {
    int (*get_area)(Shape *self, int factor);
};

static int circle_area(Shape *self, int factor) {
    (void)self;
    return factor * 2;
}

static int square_area(Shape *self, int factor) {
    (void)self;
    return factor * 3;
}

static int triangle_area(Shape *self, int factor) {
    (void)self;
    return factor * 4;
}

int main(void) {
    Shape circle = { circle_area };
    Shape square = { square_area };
    Shape triangle = { triangle_area };
    Shape *shapes[3] = { &circle, &square, &triangle };

    int total = 0;
    for (int i = 0; i < 1200000; i++) {
        Shape *s = shapes[i % 3];
        total = (total + s->get_area(s, i % 10)) & 0x7FFFFFFF;
    }

    if (total == 16200000) {
        printf("RESULT: 16200000\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
