#include "matrix_core.h"

#include <array>
#include <cstdlib>
#include <cstring>
#include <iostream>

#define CHECK_TRUE(cond)                                                           \
    do {                                                                           \
        if (!(cond)) {                                                             \
            std::cerr << "Assertion failed: " << #cond << " at " << __FILE__      \
                      << ":" << __LINE__ << "\n";                                  \
            std::abort();                                                          \
        }                                                                          \
    } while (0)

int main() {
    CHECK_TRUE(matrix_add_i32(15, 27) == 42);
    CHECK_TRUE(matrix_mul_i64(100000LL, 500000LL) == 50000000000LL);
    double fma_val = matrix_fma_f64(2.5, 4.0, 1.5);
    CHECK_TRUE(fma_val > 11.49 && fma_val < 11.51);

    const char* raw_text = "solix";
    std::array<char, 6> reversed{};
    matrix_reverse_ascii(raw_text, 5U, reversed.data());
    CHECK_TRUE(std::strcmp(reversed.data(), "xilos") == 0);

    std::array<uint8_t, 4> input_chars{'a', 'B', 'z', '1'};
    std::array<uint8_t, 4> upper_chars{};
    matrix_upper_ascii(input_chars.data(), 4U, upper_chars.data());
    CHECK_TRUE(upper_chars[0] == 'A' && upper_chars[2] == 'Z' && upper_chars[3] == '1');

    std::array<uint8_t, 3> input_bytes{10U, 20U, 30U};
    std::array<uint8_t, 3> xored_bytes{};
    matrix_xor_bytes(input_bytes.data(), 3U, 7U, xored_bytes.data());
    CHECK_TRUE(xored_bytes[0] == 13U && xored_bytes[1] == 19U && xored_bytes[2] == 25U);

    std::cout << "cmake_backend standalone CTest passed.\n";
    return 0;
}
