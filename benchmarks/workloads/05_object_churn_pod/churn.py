class Point:
    __slots__ = ("x", "y")

    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

if __name__ == "__main__":
    total = 0
    for i in range(3000000):
        p = Point(i, i * 2)
        total = (total + p.x + p.y) & 0x7FFFFFFF

    if total == 913288672:
        print("RESULT: 913288672")
    else:
        print("FAIL")
