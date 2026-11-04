class WorkerA:
    def work(self, x: int) -> int:
        return x + 5

class WorkerB:
    def work(self, x: int) -> int:
        return x * 3

if __name__ == "__main__":
    wa = WorkerA()
    wb = WorkerB()
    total = 0
    for i in range(100000):
        w = wa if (i % 2 == 0) else wb
        total = (total + w.work(i % 10)) & 0x7FFFFFFF

    if total == 1200000:
        print("RESULT: 1200000")
    else:
        print("FAIL")
