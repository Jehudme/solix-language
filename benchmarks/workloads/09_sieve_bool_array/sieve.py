if __name__ == "__main__":
    limit = 1000000
    is_prime = [True] * (limit + 1)
    is_prime[0] = False
    is_prime[1] = False

    p = 2
    while p * p <= limit:
        if is_prime[p]:
            for m in range(p * p, limit + 1, p):
                is_prime[m] = False
        p += 1

    count = sum(1 for i in range(2, limit + 1) if is_prime[i])
    if count == 78498:
        print("RESULT: 78498")
    else:
        print("FAIL")
