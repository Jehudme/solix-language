#include "matrix_core.h"

int32_t matrix_add_i32(int32_t left_val, int32_t right_val) {
    return left_val + right_val;
}

int64_t matrix_mul_i64(int64_t left_val, int64_t right_val) {
    return left_val * right_val;
}

double matrix_fma_f64(double factor_a, double factor_b, double offset_c) {
    return (factor_a * factor_b) + offset_c;
}

void matrix_reverse_ascii(const char* input_buf, size_t len, char* out_buf) {
    for (size_t idx = 0; idx < len; ++idx) {
        out_buf[idx] = input_buf[len - 1U - idx];
    }
}

void matrix_upper_ascii(const uint8_t* input_buf, size_t len, uint8_t* out_buf) {
    for (size_t idx = 0; idx < len; ++idx) {
        uint8_t byte_val = input_buf[idx];
        if (byte_val >= 'a' && byte_val <= 'z') {
            byte_val = (uint8_t)(byte_val - ('a' - 'A'));
        }
        out_buf[idx] = byte_val;
    }
}

void matrix_xor_bytes(const uint8_t* input_buf, size_t len, uint8_t mask, uint8_t* out_buf) {
    for (size_t idx = 0; idx < len; ++idx) {
        out_buf[idx] = (uint8_t)(input_buf[idx] ^ mask);
    }
}
