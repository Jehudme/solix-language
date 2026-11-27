if __name__ == "__main__":
    n = 60
    a = [(i + j) % 100 for i in range(n) for j in range(n)]
    b = [(i * j) % 100 for i in range(n) for j in range(n)]

    trace = 0
    for _ in range(50):
        for i in range(n):
            c_ii = 0
            for k in range(n):
                c_ii = (c_ii + a[i * n + k] * b[k * n + i]) & 0x7FFFFFFF
            trace = (trace + c_ii) & 0x7FFFFFFF

    if trace == 455915000:
        print("RESULT: 455915000")
    else:
        print("FAIL")
