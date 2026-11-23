#include "solix_ffi.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define TEST_CHECK(condition)                                                        \
    do {                                                                             \
        if (!(condition)) {                                                          \
            fprintf(stderr, "Test check failed: %s (%s:%d)\n", #condition, __FILE__, __LINE__); \
            abort();                                                                 \
        }                                                                            \
    } while (0)

typedef struct MockCollector {
    const char* symbols[8];
    size_t count;
} MockCollector;

static void mock_register_callback(
    void* collector_context,
    const char* symbol_name,
    SolixNativeFunctionPtr function_pointer
) {
    TEST_CHECK(function_pointer != NULL);
    MockCollector* collector = (MockCollector*)collector_context;
    TEST_CHECK(collector->count < 8U);
    collector->symbols[collector->count++] = symbol_name;
}

typedef struct MockVmState {
    int32_t field_i32_val;
    uint64_t field_u64_val;
    uint64_t array_slots[8];
    uint32_t array_length;
    uint32_t last_threshold;
    uint64_t cycles_collected;
    char stored_string[64];
} MockVmState;

static int32_t mock_get_field_i32(void* vm_ptr, uint32_t obj_addr, uint32_t field_index) {
    (void)obj_addr;
    (void)field_index;
    return ((MockVmState*)vm_ptr)->field_i32_val;
}

static void mock_set_field_i32(
    void* vm_ptr,
    uint32_t obj_addr,
    uint32_t field_index,
    int32_t value
) {
    (void)obj_addr;
    (void)field_index;
    ((MockVmState*)vm_ptr)->field_i32_val = value;
}

static uint32_t mock_read_bytes(
    void* vm_ptr,
    uint32_t src_addr,
    uint8_t* out_buf,
    uint32_t cap
) {
    (void)src_addr;
    MockVmState* state = (MockVmState*)vm_ptr;
    uint32_t len = (uint32_t)strlen(state->stored_string);
    if (out_buf != NULL && cap > 0U) {
        uint32_t copy_len = cap < len ? cap : len;
        memcpy(out_buf, state->stored_string, copy_len);
    }
    return len;
}

static uint32_t mock_alloc_string(void* vm_ptr, const uint8_t* ptr, uint32_t len) {
    MockVmState* state = (MockVmState*)vm_ptr;
    uint32_t copy_len = len < 63U ? len : 63U;
    if (ptr != NULL && copy_len > 0U) {
        memcpy(state->stored_string, ptr, copy_len);
    }
    state->stored_string[copy_len] = '\0';
    return 1024U;
}

static uint32_t mock_alloc_chars(void* vm_ptr, const uint8_t* ptr, uint32_t len) {
    (void)vm_ptr;
    (void)ptr;
    return 2048U + len;
}

static uint32_t mock_alloc_byte_array(void* vm_ptr, const uint8_t* ptr, uint32_t len) {
    (void)vm_ptr;
    (void)ptr;
    return 3072U + len;
}

static uint32_t mock_alloc_array(void* vm_ptr, uint32_t count, uint8_t tag) {
    (void)tag;
    MockVmState* state = (MockVmState*)vm_ptr;
    state->array_length = count;
    return 4096U;
}

static uint32_t mock_array_len(void* vm_ptr, uint32_t array_addr) {
    (void)array_addr;
    return ((MockVmState*)vm_ptr)->array_length;
}

static uint64_t mock_array_get_u64(void* vm_ptr, uint32_t array_addr, uint32_t elem_index) {
    (void)array_addr;
    MockVmState* state = (MockVmState*)vm_ptr;
    return elem_index < 8U ? state->array_slots[elem_index] : 0U;
}

static void mock_array_set_u64(
    void* vm_ptr,
    uint32_t array_addr,
    uint32_t elem_index,
    uint64_t raw_val
) {
    (void)array_addr;
    MockVmState* state = (MockVmState*)vm_ptr;
    if (elem_index < 8U) {
        state->array_slots[elem_index] = raw_val;
    }
}

static uint32_t mock_alloc_object(void* vm_ptr, uint32_t vtable_id, uint32_t field_count) {
    (void)vm_ptr;
    (void)field_count;
    return vtable_id == 77U ? 8192U : 0U;
}

static uint64_t mock_read_field_u64(void* vm_ptr, uint32_t obj_addr, uint32_t offset) {
    (void)obj_addr;
    (void)offset;
    return ((MockVmState*)vm_ptr)->field_u64_val;
}

static void mock_write_field_u64(
    void* vm_ptr,
    uint32_t obj_addr,
    uint32_t offset,
    uint64_t raw_val
) {
    (void)obj_addr;
    (void)offset;
    ((MockVmState*)vm_ptr)->field_u64_val = raw_val;
}

static uint32_t mock_get_class_vtable_id(void* vm_ptr, const char* name) {
    (void)vm_ptr;
    return (name != NULL && strcmp(name, "Point") == 0) ? 77U : 0U;
}

static uint64_t mock_get_vm_metric(void* vm_ptr, uint32_t metric_id) {
    (void)vm_ptr;
    return (uint64_t)metric_id * 100U;
}

static uint64_t mock_collect_cycles(void* vm_ptr) {
    MockVmState* state = (MockVmState*)vm_ptr;
    state->cycles_collected += 1U;
    return state->cycles_collected;
}

static void mock_set_cycle_threshold(void* vm_ptr, uint32_t threshold) {
    ((MockVmState*)vm_ptr)->last_threshold = threshold;
}

static SolixCValue native_sample_add(
    void* vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    if (argument_count < 2U || arguments == NULL) {
        solix_vm_set_error(vm_context, out_error, "missing operands");
        return solix_cvalue_null();
    }
    return solix_cvalue_i32(solix_cvalue_as_i32(arguments[0]) + solix_cvalue_as_i32(arguments[1]));
}

static const SolixNativeEntry TEST_ENTRIES[] = {
    { "CTestBridge::add", native_sample_add },
};

SOLIX_REGISTER_MODULE(TEST_ENTRIES)

int main(void) {
    /* 0. Verify C-FFI Version Handshake */
    TEST_CHECK(SOLIX_FFI_VERSION_MAJOR == 2);
    TEST_CHECK(SOLIX_FFI_VERSION_MINOR == 0);
    TEST_CHECK(SOLIX_FFI_VERSION_PATCH == 0);
    TEST_CHECK(strcmp(SOLIX_FFI_VERSION, "2.0.0") == 0);

    /* 1. Verify SolixCValue encodings */
    TEST_CHECK(solix_cvalue_is_null(solix_cvalue_null()));
    TEST_CHECK(solix_cvalue_as_bool(solix_cvalue_bool(true)) == true);
    TEST_CHECK(solix_cvalue_as_bool(solix_cvalue_bool(false)) == false);
    TEST_CHECK(solix_cvalue_as_i32(solix_cvalue_i32(-42)) == -42);
    TEST_CHECK(solix_cvalue_as_i64(solix_cvalue_i64(900000000000LL)) == 900000000000LL);
    TEST_CHECK(fabs(solix_cvalue_as_f64(solix_cvalue_f64(3.1415926)) - 3.1415926) < 1e-12);
    TEST_CHECK(solix_cvalue_as_ref(solix_cvalue_ref(4096U)) == 4096U);
    TEST_CHECK(solix_cvalue_as_fnptr(solix_cvalue_fnptr(17U)) == 17U);

    /* 2. Verify SolixVmContext C-ABI layout size (19 pointers/u64 fields = 152 bytes on 64-bit) */
    TEST_CHECK(sizeof(SolixVmContext) == sizeof(void*) * 19U);

    /* 3. Verify SolixVmContext helpers end-to-end */
    MockVmState state;
    memset(&state, 0, sizeof(state));
    SolixVmContext vm_ctx = {
        SOLIX_VM_CONTEXT_MAGIC,
        &state,
        mock_get_field_i32,
        mock_set_field_i32,
        mock_read_bytes,
        mock_alloc_string,
        mock_alloc_chars,
        mock_alloc_byte_array,
        mock_alloc_array,
        mock_array_len,
        mock_array_get_u64,
        mock_array_set_u64,
        mock_alloc_object,
        mock_read_field_u64,
        mock_write_field_u64,
        mock_get_class_vtable_id,
        mock_get_vm_metric,
        mock_collect_cycles,
        mock_set_cycle_threshold,
    };

    const SolixVmContext* verified = solix_vm_context_from_ptr(&vm_ctx);
    TEST_CHECK(verified != NULL);

    uint32_t str_addr = solix_vm_alloc_string(verified, "hello-c-ffi");
    TEST_CHECK(str_addr == 1024U);
    size_t read_len = 0;
    char* text = solix_vm_read_string_alloc(verified, str_addr, &read_len);
    TEST_CHECK(text != NULL && read_len == 11U && strcmp(text, "hello-c-ffi") == 0);
    free(text);

    TEST_CHECK(solix_vm_alloc_chars(verified, "abc", 3U) == 2051U);
    TEST_CHECK(solix_vm_alloc_byte_array(verified, (const uint8_t*)"xy", 2U) == 3074U);

    uint32_t arr_addr = solix_vm_alloc_array(verified, 4U, SOLIX_ARRAY_TAG_I64);
    TEST_CHECK(solix_vm_array_len(verified, arr_addr) == 4U);
    solix_vm_array_set_u64(verified, arr_addr, 2U, UINT64_C(98765));
    TEST_CHECK(solix_vm_array_get_u64(verified, arr_addr, 2U) == UINT64_C(98765));

    uint32_t vtable_id = solix_vm_get_class_vtable_id(verified, "Point");
    uint32_t obj_addr = solix_vm_alloc_object(verified, vtable_id, 2U);
    TEST_CHECK(obj_addr == 8192U);
    solix_vm_set_field_i32(verified, obj_addr, 0U, -123);
    TEST_CHECK(solix_vm_get_field_i32(verified, obj_addr, 0U) == -123);
    solix_vm_write_field_u64(verified, obj_addr, 16U, UINT64_C(555666));
    TEST_CHECK(solix_vm_read_field_u64(verified, obj_addr, 16U) == UINT64_C(555666));

    TEST_CHECK(solix_vm_get_metric(verified, 3U) == UINT64_C(300));
    solix_vm_set_cycle_threshold(verified, 512U);
    TEST_CHECK(state.last_threshold == 512U);
    TEST_CHECK(solix_vm_collect_cycles(verified) == 1U);

    /* 4. Verify registration + callback error signaling */
    MockCollector collector = { { NULL }, 0U };
    solix_register_natives(&collector, mock_register_callback);
    TEST_CHECK(collector.count == 1U);
    TEST_CHECK(strcmp(collector.symbols[0], "CTestBridge::add") == 0);

    SolixCValue err = solix_cvalue_null();
    SolixCValue args[2] = { solix_cvalue_i32(20), solix_cvalue_i32(22) };
    SolixCValue sum = native_sample_add(&vm_ctx, args, 2U, &err);
    TEST_CHECK(solix_cvalue_as_i32(sum) == 42);
    TEST_CHECK(solix_cvalue_is_null(err));

    SolixCValue missing_err = solix_cvalue_null();
    (void)native_sample_add(&vm_ctx, args, 1U, &missing_err);
    TEST_CHECK(!solix_cvalue_is_null(missing_err));

    printf("C11 solix_ffi tests passed.\n");
    return 0;
}
