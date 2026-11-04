import sys

sys.setrecursionlimit(5000)

def ack(m: int, n: int) -> int:
    if m == 0:
        return n + 1
    if n == 0:
        return ack(m - 1, 1)
    return ack(m - 1, ack(m, n - 1))

if __name__ == "__main__":
    result = ack(3, 6)
    if result == 509:
        print("RESULT: 509")
    else:
        print("FAIL")
