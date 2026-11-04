def process(s: str, factor: int) -> int:
    if s == "alpha":
        return factor * 2
    if s == "beta":
        return factor * 3
    if s == "gamma":
        return factor * 5
    return factor

if __name__ == "__main__":
    words = ["alpha", "beta", "gamma"]
    total = 0
    for i in range(100000):
        w = words[i % 3]
        total = (total + process(w, i % 20)) & 0x7FFFFFFF

    if total == 3166647:
        print("RESULT: 3166647")
    else:
        print("FAIL")
