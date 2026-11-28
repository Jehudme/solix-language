from typing import Generic, TypeVar

T = TypeVar("T")

class Box(Generic[T]):
    __slots__ = ("value",)

    def __init__(self, value: T):
        self.value = value

    def get_val(self) -> T:
        return self.value

if __name__ == "__main__":
    total = 0
    for i in range(800000):
        b1 = Box(i)
        b2 = Box(i * 4 + 7)
        total = (total + b1.get_val() + b2.get_val()) & 0x7FFFFFFF

    if total == 128282240:
        print("RESULT: 128282240")
    else:
        print("FAIL")
