if __name__ == "__main__":
    base = 7
    adder = lambda n: n + base

    total = 0
    for i in range(4000000):
        total = (total + adder(i % 100)) & 0x7FFFFFFF

    if total == 226000000:
        print("RESULT: 226000000")
    else:
        print("FAIL")
