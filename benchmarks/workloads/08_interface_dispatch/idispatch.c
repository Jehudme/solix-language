#include <stdio.h>

typedef struct IWorker IWorker;
struct IWorker {
    int (*work)(IWorker *self, int x);
};

static int worker_a_work(IWorker *self, int x) {
    (void)self;
    return x + 5;
}

static int worker_b_work(IWorker *self, int x) {
    (void)self;
    return x * 3;
}

int main(void) {
    IWorker wa = { worker_a_work };
    IWorker wb = { worker_b_work };

    int total = 0;
    for (int i = 0; i < 100000; i++) {
        IWorker *w = (i % 2 == 0) ? &wa : &wb;
        total = (total + w->work(w, i % 10)) & 0x7FFFFFFF;
    }

    if (total == 1200000) {
        printf("RESULT: 1200000\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
