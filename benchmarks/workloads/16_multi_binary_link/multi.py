def compute(x: int, y: int) -> int:
    return x * 3 + y * 7

if __name__ == "__main__":
    total = 0
    for i in range(3000000):
        total = (total + compute(i % 100, i % 50)) & 0x7FFFFFFF

    if total == 960000000:
        print("RESULT: 960000000")
    else:
        print("FAIL")
