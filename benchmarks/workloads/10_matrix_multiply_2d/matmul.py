if __name__ == "__main__":
    n = 60
    a = [(i + j) % 100 for i in range(n) for j in range(n)]
    b = [(i * j) % 100 for i in range(n) for j in range(n)]

    trace = 0
    for i in range(n):
        c_ii = 0
        for k in range(n):
            c_ii = (c_ii + a[i * n + k] * b[k * n + i]) & 0x7FFFFFFF
        trace = (trace + c_ii) & 0x7FFFFFFF

    if trace == 9118300:
        print("RESULT: 9118300")
    else:
        print("FAIL")
