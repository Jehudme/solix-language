def compute(x: int, y: int) -> int:
    return x * 3 + y * 7

if __name__ == "__main__":
    total = 0
    for i in range(50000):
        total = (total + compute(i % 100, i % 50)) & 0x7FFFFFFF

    if total == 16000000:
        print("RESULT: 16000000")
    else:
        print("FAIL")
