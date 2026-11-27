#include "solix_ffi.h"
#include "matrix_core.h"

#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

namespace {

SolixCValue native_echo_null(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)raw_vm_context;
    (void)out_error;
    if (argument_count == 0U || arguments == nullptr || solix_cvalue_is_null(arguments[0])) {
        return solix_cvalue_null();
    }
    return arguments[0];
}

SolixCValue native_invert_bool(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)raw_vm_context;
    (void)out_error;
    if (argument_count < 1U || arguments == nullptr) {
        return solix_cvalue_bool(false);
    }
    return solix_cvalue_bool(!solix_cvalue_as_bool(arguments[0]));
}

SolixCValue native_add_i32(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)raw_vm_context;
    (void)out_error;
    if (argument_count < 2U || arguments == nullptr) {
        return solix_cvalue_i32(0);
    }
    return solix_cvalue_i32(
        matrix_add_i32(solix_cvalue_as_i32(arguments[0]), solix_cvalue_as_i32(arguments[1]))
    );
}

SolixCValue native_mul_i64(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)raw_vm_context;
    (void)out_error;
    if (argument_count < 2U || arguments == nullptr) {
        return solix_cvalue_i64(0);
    }
    return solix_cvalue_i64(
        matrix_mul_i64(solix_cvalue_as_i64(arguments[0]), solix_cvalue_as_i64(arguments[1]))
    );
}

SolixCValue native_fma_f64(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)raw_vm_context;
    (void)out_error;
    if (argument_count < 3U || arguments == nullptr) {
        return solix_cvalue_f64(0.0);
    }
    return solix_cvalue_f64(matrix_fma_f64(
        solix_cvalue_as_f64(arguments[0]),
        solix_cvalue_as_f64(arguments[1]),
        solix_cvalue_as_f64(arguments[2])
    ));
}

SolixCValue native_reverse_string(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr || argument_count < 1U || arguments == nullptr) {
        solix_vm_set_error(raw_vm_context, out_error, "reverse_string requires valid VM context");
        return solix_cvalue_null();
    }
    size_t text_len = 0U;
    char* input_text = solix_vm_read_string_alloc(ctx, solix_cvalue_as_ref(arguments[0]), &text_len);
    if (input_text == nullptr) {
        return solix_cvalue_ref(solix_vm_alloc_string(ctx, ""));
    }
    std::string reversed(text_len, '\0');
    matrix_reverse_ascii(input_text, text_len, reversed.data());
    std::free(input_text);
    return solix_cvalue_ref(
        solix_vm_alloc_string_len(ctx, reversed.data(), static_cast<uint32_t>(text_len))
    );
}

SolixCValue native_upper_chars(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)out_error;
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr || argument_count < 1U || arguments == nullptr) {
        return solix_cvalue_null();
    }
    uint32_t char_addr = solix_cvalue_as_ref(arguments[0]);
    uint32_t len = solix_vm_read_bytes(ctx, char_addr, nullptr, 0U);
    std::vector<uint8_t> raw_chars(len);
    std::vector<uint8_t> upper_chars(len);
    if (len > 0U) {
        solix_vm_read_bytes(ctx, char_addr, raw_chars.data(), len);
        matrix_upper_ascii(raw_chars.data(), len, upper_chars.data());
    }
    uint32_t out_addr = solix_vm_alloc_chars(
        ctx,
        reinterpret_cast<const char*>(upper_chars.data()),
        len
    );
    return solix_cvalue_ref(out_addr);
}

SolixCValue native_xor_bytes(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)out_error;
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr || argument_count < 2U || arguments == nullptr) {
        return solix_cvalue_null();
    }
    uint32_t bytes_addr = solix_cvalue_as_ref(arguments[0]);
    uint8_t mask = static_cast<uint8_t>(solix_cvalue_as_i32(arguments[1]) & 0xFF);
    uint32_t len = solix_vm_read_bytes(ctx, bytes_addr, nullptr, 0U);
    std::vector<uint8_t> raw_bytes(len);
    std::vector<uint8_t> out_bytes(len);
    if (len > 0U) {
        solix_vm_read_bytes(ctx, bytes_addr, raw_bytes.data(), len);
        matrix_xor_bytes(raw_bytes.data(), len, mask, out_bytes.data());
    }
    return solix_cvalue_ref(solix_vm_alloc_byte_array(ctx, out_bytes.data(), len));
}

SolixCValue native_scale_i64_array(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)out_error;
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr || argument_count < 2U || arguments == nullptr) {
        return solix_cvalue_null();
    }
    uint32_t src_addr = solix_cvalue_as_ref(arguments[0]);
    int64_t scale_factor = solix_cvalue_as_i64(arguments[1]);
    uint32_t count = solix_vm_array_len(ctx, src_addr);
    uint32_t dst_addr = solix_vm_alloc_array(ctx, count, SOLIX_ARRAY_TAG_I64);
    for (uint32_t idx = 0U; idx < count; ++idx) {
        int64_t elem_val = static_cast<int64_t>(solix_vm_array_get_u64(ctx, src_addr, idx));
        int64_t scaled = elem_val * scale_factor;
        solix_vm_array_set_u64(ctx, dst_addr, idx, static_cast<uint64_t>(scaled));
    }
    return solix_cvalue_ref(dst_addr);
}

SolixCValue native_sum_f64_array(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)out_error;
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr || argument_count < 1U || arguments == nullptr) {
        return solix_cvalue_f64(0.0);
    }
    uint32_t src_addr = solix_cvalue_as_ref(arguments[0]);
    uint32_t count = solix_vm_array_len(ctx, src_addr);
    double total = 0.0;
    for (uint32_t idx = 0U; idx < count; ++idx) {
        uint64_t raw_bits = solix_vm_array_get_u64(ctx, src_addr, idx);
        double decoded = 0.0;
        std::memcpy(&decoded, &raw_bits, sizeof(decoded));
        total += decoded;
    }
    return solix_cvalue_f64(total);
}

SolixCValue native_make_pair_strings(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)out_error;
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr || argument_count < 2U || arguments == nullptr) {
        return solix_cvalue_null();
    }
    size_t len_a = 0U;
    size_t len_b = 0U;
    char* text_a = solix_vm_read_string_alloc(ctx, solix_cvalue_as_ref(arguments[0]), &len_a);
    char* text_b = solix_vm_read_string_alloc(ctx, solix_cvalue_as_ref(arguments[1]), &len_b);

    uint32_t str_a = solix_vm_alloc_string(ctx, text_a != nullptr ? text_a : "");
    uint32_t str_b = solix_vm_alloc_string(ctx, text_b != nullptr ? text_b : "");
    std::free(text_a);
    std::free(text_b);

    uint32_t pair_arr = solix_vm_alloc_array(ctx, 2U, SOLIX_ARRAY_TAG_REF);
    solix_vm_array_set_u64(ctx, pair_arr, 0U, static_cast<uint64_t>(str_a));
    solix_vm_array_set_u64(ctx, pair_arr, 1U, static_cast<uint64_t>(str_b));
    return solix_cvalue_ref(pair_arr);
}

SolixCValue native_create_box(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr || argument_count < 3U || arguments == nullptr) {
        solix_vm_set_error(raw_vm_context, out_error, "create_box expects 3 arguments");
        return solix_cvalue_null();
    }
    uint32_t vtable_id = solix_vm_get_class_vtable_id(ctx, "MatrixBox");
    if (vtable_id == 0U) {
        solix_vm_set_error(raw_vm_context, out_error, "MatrixBox class not found");
        return solix_cvalue_null();
    }
    uint32_t box_addr = solix_vm_alloc_object(ctx, vtable_id, 3U);
    solix_vm_set_field_i32_by_index(ctx, box_addr, 0U, solix_cvalue_as_i32(arguments[0]));
    solix_vm_write_field_u64_by_index(
        ctx,
        box_addr,
        1U,
        static_cast<uint64_t>(solix_cvalue_as_i64(arguments[1]))
    );
    solix_vm_write_field_u64_by_index(
        ctx,
        box_addr,
        2U,
        static_cast<uint64_t>(solix_cvalue_as_ref(arguments[2]))
    );
    return solix_cvalue_ref(box_addr);
}

SolixCValue native_mutate_box(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)out_error;
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr || argument_count < 2U || arguments == nullptr) {
        return solix_cvalue_i64(0);
    }
    uint32_t box_addr = solix_cvalue_as_ref(arguments[0]);
    int32_t delta = solix_cvalue_as_i32(arguments[1]);
    int32_t prev_tag = solix_vm_get_field_i32_by_index(ctx, box_addr, 0U);
    int32_t next_tag = prev_tag + delta;
    solix_vm_set_field_i32_by_index(ctx, box_addr, 0U, next_tag);
    int64_t weight = static_cast<int64_t>(solix_vm_read_field_u64_by_index(ctx, box_addr, 1U));
    return solix_cvalue_i64(static_cast<int64_t>(next_tag) + weight);
}

SolixCValue native_exercise_gc_and_metrics(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    (void)arguments;
    (void)argument_count;
    (void)out_error;
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx == nullptr) {
        return solix_cvalue_i64(-1);
    }
    solix_vm_set_cycle_threshold(ctx, 128U);
    uint64_t metric_val = solix_vm_get_metric(ctx, 0U);
    uint64_t collected = solix_vm_collect_cycles(ctx);
    return solix_cvalue_i64(static_cast<int64_t>(metric_val + collected));
}

SolixCValue native_checked_div(
    void* raw_vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    if (argument_count < 2U || arguments == nullptr) {
        solix_vm_set_error(raw_vm_context, out_error, "checked_div requires 2 arguments");
        return solix_cvalue_i32(0);
    }
    int32_t numerator = solix_cvalue_as_i32(arguments[0]);
    int32_t denominator = solix_cvalue_as_i32(arguments[1]);
    if (denominator == 0) {
        solix_vm_set_error(raw_vm_context, out_error, "division by zero in native backend");
        return solix_cvalue_i32(0);
    }
    return solix_cvalue_i32(numerator / denominator);
}

const SolixNativeEntry CMAKE_MATRIX_ENTRIES[] = {
    {"FfiMatrix::echo_null", native_echo_null},
    {"FfiMatrix::invert_bool", native_invert_bool},
    {"FfiMatrix::add_i32", native_add_i32},
    {"FfiMatrix::mul_i64", native_mul_i64},
    {"FfiMatrix::fma_f64", native_fma_f64},
    {"FfiMatrix::reverse_string", native_reverse_string},
    {"FfiMatrix::upper_chars", native_upper_chars},
    {"FfiMatrix::xor_bytes", native_xor_bytes},
    {"FfiMatrix::scale_i64_array", native_scale_i64_array},
    {"FfiMatrix::sum_f64_array", native_sum_f64_array},
    {"FfiMatrix::make_pair_strings", native_make_pair_strings},
    {"FfiMatrix::create_box", native_create_box},
    {"FfiMatrix::mutate_box", native_mutate_box},
    {"FfiMatrix::exercise_gc_and_metrics", native_exercise_gc_and_metrics},
    {"FfiMatrix::checked_div", native_checked_div},
};

} // namespace

SOLIX_REGISTER_MODULE(CMAKE_MATRIX_ENTRIES)
