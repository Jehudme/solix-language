#include <stdio.h>

int main(void) {
    double x = 0.5;
    double y = 1.2;
    double vx = 0.01;
    double vy = 0.02;

    for (int i = 0; i < 50000; i++) {
        double dist_sq = x * x + y * y + 0.1;
        double force = 1.0 / dist_sq;
        vx += force * 0.001;
        vy += force * 0.001;
        x += vx;
        y += vy;
    }

    int energy = (int)(x * 100.0) + (int)(y * 100.0);
    if (energy == 358036) {
        printf("RESULT: 358036\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
