if __name__ == "__main__":
    total = 0
    for i in range(1000000):
        total = (total + (i ^ (i % 7))) & 0x7FFFFFFF

    if total == 1783293643:
        print("RESULT: 1783293643")
    else:
        print("FAIL")
