#ifndef MATRIX_CORE_H
#define MATRIX_CORE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

int32_t matrix_add_i32(int32_t left_val, int32_t right_val);
int64_t matrix_mul_i64(int64_t left_val, int64_t right_val);
double matrix_fma_f64(double factor_a, double factor_b, double offset_c);
void matrix_reverse_ascii(const char* input_buf, size_t len, char* out_buf);
void matrix_upper_ascii(const uint8_t* input_buf, size_t len, uint8_t* out_buf);
void matrix_xor_bytes(const uint8_t* input_buf, size_t len, uint8_t mask, uint8_t* out_buf);

#ifdef __cplusplus
}
#endif

#endif /* MATRIX_CORE_H */
