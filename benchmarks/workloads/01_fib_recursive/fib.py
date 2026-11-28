def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

if __name__ == "__main__":
    result = fib(32)
    if result == 2178309:
        print("RESULT: 2178309")
    else:
        print("FAIL")
