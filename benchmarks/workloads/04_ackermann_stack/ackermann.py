import sys

sys.setrecursionlimit(5000)

def ack(m: int, n: int) -> int:
    if m == 0:
        return n + 1
    if n == 0:
        return ack(m - 1, 1)
    return ack(m - 1, ack(m, n - 1))

if __name__ == "__main__":
    total = 0
    for _ in range(15):
        total = (total + ack(3, 6)) & 0x7FFFFFFF
    if total == 7635:
        print("RESULT: 7635")
    else:
        print("FAIL")
