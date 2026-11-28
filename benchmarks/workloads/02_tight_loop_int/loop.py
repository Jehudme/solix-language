if __name__ == "__main__":
    total = 0
    for i in range(8000000):
        total = (total + (i ^ (i % 7))) & 0x7FFFFFFF

    if total == 342161131:
        print("RESULT: 342161131")
    else:
        print("FAIL")
