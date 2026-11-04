class Point:
    __slots__ = ("x", "y")

    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

if __name__ == "__main__":
    total = 0
    for i in range(50000):
        p = Point(i, i * 2)
        total = (total + p.x + p.y) & 0x7FFFFFFF

    if total == 1602441352:
        print("RESULT: 1602441352")
    else:
        print("FAIL")
