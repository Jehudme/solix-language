def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

if __name__ == "__main__":
    result = fib(28)
    if result == 317811:
        print("RESULT: 317811")
    else:
        print("FAIL")
