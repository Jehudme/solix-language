class Shape:
    def get_area(self, factor: int) -> int:
        return factor

class Circle(Shape):
    def get_area(self, factor: int) -> int:
        return factor * 2

class Square(Shape):
    def get_area(self, factor: int) -> int:
        return factor * 3

class Triangle(Shape):
    def get_area(self, factor: int) -> int:
        return factor * 4

if __name__ == "__main__":
    shapes = [Circle(), Square(), Triangle()]
    total = 0
    for i in range(100000):
        total = (total + shapes[i % 3].get_area(i % 10)) & 0x7FFFFFFF

    if total == 1349997:
        print("RESULT: 1349997")
    else:
        print("FAIL")
