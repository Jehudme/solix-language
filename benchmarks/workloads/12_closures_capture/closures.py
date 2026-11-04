if __name__ == "__main__":
    base = 7
    adder = lambda n: n + base

    total = 0
    for i in range(50000):
        total = (total + adder(i % 100)) & 0x7FFFFFFF

    if total == 2825000:
        print("RESULT: 2825000")
    else:
        print("FAIL")
