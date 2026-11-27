class MyException(Exception):
    def __init__(self, code: int):
        self.code = code

if __name__ == "__main__":
    caught_count = 0
    normal_sum = 0

    for i in range(250000):
        try:
            normal_sum = (normal_sum + (i % 10)) & 0x7FFFFFFF
        except MyException:
            caught_count += 1

    for i in range(2500):
        try:
            raise MyException(i)
        except MyException as ex:
            caught_count += ex.code

    checksum = (normal_sum + caught_count) & 0x7FFFFFFF
    if checksum == 4248750:
        print("RESULT: 4248750")
    else:
        print("FAIL")
