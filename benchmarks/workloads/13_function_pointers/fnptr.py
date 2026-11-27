def fn1(x: int) -> int:
    return x * 2 + 1

def fn2(x: int) -> int:
    return x * 3 - 2

def fn3(x: int) -> int:
    return x * 5 + 4

if __name__ == "__main__":
    total = 0
    for i in range(400000):
        idx = i % 3
        f = fn1 if idx == 0 else (fn2 if idx == 1 else fn3)
        total = (total + f(i % 50)) & 0x7FFFFFFF

    if total == 33066617:
        print("RESULT: 33066617")
    else:
        print("FAIL")
