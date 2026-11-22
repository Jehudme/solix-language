/**
 * @file solix_ffi.h
 * @brief Official C11 / C++17 Foreign Function Interface (FFI) SDK for Solix.
 *
 * Provides the stable 16-byte C-ABI tagged value (`SolixCValue`), VM context
 * accessor table (`SolixVmContext`), safe marshaling helpers, and symbol
 * export macros (`SOLIX_EXPORT_NATIVES_BEGIN` / `SOLIX_REGISTER_MODULE`)
 * matching `ffi/rust` (`solix-ffi`) and `crates/solix-runtime/src/native/ffi.rs`.
 */

#ifndef SOLIX_FFI_H
#define SOLIX_FFI_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define SOLIX_FFI_VERSION_MAJOR 1
#define SOLIX_FFI_VERSION_MINOR 8
#define SOLIX_FFI_VERSION_PATCH 0
#define SOLIX_FFI_VERSION "1.8.0"

#ifdef __cplusplus
extern "C" {
#endif

/* ------------------------------------------------------------------------- */
/* Visibility & Calling Convention Macros                                    */
/* ------------------------------------------------------------------------- */

#if defined(_WIN32) || defined(__CYGWIN__)
#define SOLIX_FFI_EXPORT __declspec(dllexport)
#elif defined(__GNUC__) || defined(__clang__)
#define SOLIX_FFI_EXPORT __attribute__((visibility("default")))
#else
#define SOLIX_FFI_EXPORT
#endif

#ifdef __cplusplus
#define SOLIX_FFI_EXTERN_C extern "C"
#else
#define SOLIX_FFI_EXTERN_C
#endif

/* ------------------------------------------------------------------------- */
/* Value Tags & Array Type Tags                                              */
/* ------------------------------------------------------------------------- */

#define SOLIX_TAG_NULL  UINT32_C(0)
#define SOLIX_TAG_BOOL  UINT32_C(1)
#define SOLIX_TAG_I32   UINT32_C(2)
#define SOLIX_TAG_I64   UINT32_C(3)
#define SOLIX_TAG_F64   UINT32_C(4)
#define SOLIX_TAG_REF   UINT32_C(5)
#define SOLIX_TAG_FNPTR UINT32_C(6)

#define SOLIX_ARRAY_TAG_I8   UINT8_C(1)
#define SOLIX_ARRAY_TAG_I64  UINT8_C(4)
#define SOLIX_ARRAY_TAG_F32  UINT8_C(9)
#define SOLIX_ARRAY_TAG_F64  UINT8_C(10)
#define SOLIX_ARRAY_TAG_CHAR UINT8_C(12)
#define SOLIX_ARRAY_TAG_REF  UINT8_C(13)

#define SOLIX_VM_CONTEXT_MAGIC UINT64_C(0x534F4C4958564D43) /* "SOLIXVMC" */

/* ------------------------------------------------------------------------- */
/* Stable C-ABI Tagged Value                                                 */
/* ------------------------------------------------------------------------- */

/**
 * @brief Stable 16-byte aligned C-ABI tagged value exchanged with the Solix VM.
 */
typedef struct SolixCValue {
    uint32_t tag;
    uint32_t flags;
    uint64_t payload;
} SolixCValue;

#if defined(__cplusplus)
static_assert(sizeof(SolixCValue) == 16, "SolixCValue must be 16 bytes");
static_assert(offsetof(SolixCValue, tag) == 0, "SolixCValue.tag offset mismatch");
static_assert(offsetof(SolixCValue, flags) == 4, "SolixCValue.flags offset mismatch");
static_assert(offsetof(SolixCValue, payload) == 8, "SolixCValue.payload offset mismatch");
#elif defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
_Static_assert(sizeof(SolixCValue) == 16, "SolixCValue must be 16 bytes");
_Static_assert(offsetof(SolixCValue, tag) == 0, "SolixCValue.tag offset mismatch");
_Static_assert(offsetof(SolixCValue, flags) == 4, "SolixCValue.flags offset mismatch");
_Static_assert(offsetof(SolixCValue, payload) == 8, "SolixCValue.payload offset mismatch");
#endif

/* ------------------------------------------------------------------------- */
/* C-ABI Callback Signatures                                                 */
/* ------------------------------------------------------------------------- */

typedef SolixCValue (*SolixNativeFunctionPtr)(
    void* vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
);

typedef SolixNativeFunctionPtr NativeCFunctionPtr;

typedef void (*SolixRegisterCallback)(
    void* collector_context,
    const char* symbol_name,
    SolixNativeFunctionPtr function_pointer
);

typedef SolixRegisterCallback RegisterNativeCallback;

typedef struct SolixNativeEntry {
    const char* symbol;
    SolixNativeFunctionPtr function_pointer;
} SolixNativeEntry;

/* ------------------------------------------------------------------------- */
/* VM Context Function Pointer Table                                         */
/* ------------------------------------------------------------------------- */

/**
 * @brief C-ABI function pointer table passed as `vm_context` to native callbacks.
 *
 * Field layout strictly matches `crates/solix-runtime/src/native/ffi.rs` and
 * `ffi/rust/src/context.rs`.
 */
typedef struct SolixVmContext {
    uint64_t magic;
    void* vm_ptr;
    int32_t (*get_field_i32)(void* vm_ptr, uint32_t object_address, uint32_t field_index);
    void (*set_field_i32)(
        void* vm_ptr,
        uint32_t object_address,
        uint32_t field_index,
        int32_t field_value
    );
    uint32_t (*read_bytes)(
        void* vm_ptr,
        uint32_t source_address,
        uint8_t* out_buffer,
        uint32_t capacity
    );
    uint32_t (*alloc_string)(void* vm_ptr, const uint8_t* source_pointer, uint32_t byte_count);
    uint32_t (*alloc_chars)(void* vm_ptr, const uint8_t* source_pointer, uint32_t char_count);
    uint32_t (*alloc_byte_array)(void* vm_ptr, const uint8_t* source_pointer, uint32_t byte_count);
    uint32_t (*alloc_array)(void* vm_ptr, uint32_t element_count, uint8_t type_tag);
    uint32_t (*array_len)(void* vm_ptr, uint32_t array_address);
    uint64_t (*array_get_u64)(void* vm_ptr, uint32_t array_address, uint32_t element_index);
    void (*array_set_u64)(
        void* vm_ptr,
        uint32_t array_address,
        uint32_t element_index,
        uint64_t raw_value
    );
    uint32_t (*alloc_object)(void* vm_ptr, uint32_t class_vtable_id, uint32_t field_count);
    uint64_t (*read_field_u64)(void* vm_ptr, uint32_t object_address, uint32_t field_offset);
    void (*write_field_u64)(
        void* vm_ptr,
        uint32_t object_address,
        uint32_t field_offset,
        uint64_t raw_value
    );
    uint32_t (*get_class_vtable_id)(void* vm_ptr, const char* class_name);
    uint64_t (*get_vm_metric)(void* vm_ptr, uint32_t metric_id);
    uint64_t (*collect_cycles)(void* vm_ptr);
    void (*set_cycle_threshold)(void* vm_ptr, uint32_t threshold);
} SolixVmContext;

/* ------------------------------------------------------------------------- */
/* SolixCValue Constructors & Extractors                                     */
/* ------------------------------------------------------------------------- */

static inline SolixCValue solix_cvalue_null(void) {
    SolixCValue value;
    value.tag = SOLIX_TAG_NULL;
    value.flags = 0;
    value.payload = 0;
    return value;
}

static inline SolixCValue solix_cvalue_bool(bool truth) {
    SolixCValue value;
    value.tag = SOLIX_TAG_BOOL;
    value.flags = 0;
    value.payload = truth ? UINT64_C(1) : UINT64_C(0);
    return value;
}

static inline SolixCValue solix_cvalue_i32(int32_t int_value) {
    SolixCValue value;
    value.tag = SOLIX_TAG_I32;
    value.flags = 0;
    value.payload = (uint64_t)(int64_t)int_value;
    return value;
}

static inline SolixCValue solix_cvalue_i64(int64_t int_value) {
    SolixCValue value;
    value.tag = SOLIX_TAG_I64;
    value.flags = 0;
    value.payload = (uint64_t)int_value;
    return value;
}

static inline SolixCValue solix_cvalue_f64(double float_value) {
    SolixCValue value;
    uint64_t encoded_bits = 0;
    memcpy(&encoded_bits, &float_value, sizeof(encoded_bits));
    value.tag = SOLIX_TAG_F64;
    value.flags = 0;
    value.payload = encoded_bits;
    return value;
}

static inline SolixCValue solix_cvalue_ref(uint32_t heap_address) {
    SolixCValue value;
    value.tag = SOLIX_TAG_REF;
    value.flags = 0;
    value.payload = (uint64_t)heap_address;
    return value;
}

static inline SolixCValue solix_cvalue_fnptr(uint32_t func_id) {
    SolixCValue value;
    value.tag = SOLIX_TAG_FNPTR;
    value.flags = 0;
    value.payload = (uint64_t)func_id;
    return value;
}

static inline bool solix_cvalue_is_null(SolixCValue value) {
    return value.tag == SOLIX_TAG_NULL || (value.tag == SOLIX_TAG_REF && value.payload == 0);
}

static inline bool solix_cvalue_as_bool(SolixCValue value) {
    return value.tag == SOLIX_TAG_BOOL && value.payload != 0;
}

static inline int32_t solix_cvalue_as_i32(SolixCValue value) {
    return (int32_t)(int64_t)value.payload;
}

static inline int64_t solix_cvalue_as_i64(SolixCValue value) {
    return (int64_t)value.payload;
}

static inline double solix_cvalue_as_f64(SolixCValue value) {
    double decoded = 0.0;
    memcpy(&decoded, &value.payload, sizeof(decoded));
    return decoded;
}

static inline uint32_t solix_cvalue_as_ref(SolixCValue value) {
    return (uint32_t)value.payload;
}

static inline uint32_t solix_cvalue_as_fnptr(SolixCValue value) {
    return (uint32_t)value.payload;
}

/* ------------------------------------------------------------------------- */
/* SolixVmContext Accessor Helpers                                           */
/* ------------------------------------------------------------------------- */

static inline const SolixVmContext* solix_vm_context_from_ptr(void* raw_vm_context) {
    if (raw_vm_context == NULL) {
        return NULL;
    }
    const SolixVmContext* ctx = (const SolixVmContext*)raw_vm_context;
    if (ctx->magic != SOLIX_VM_CONTEXT_MAGIC || ctx->vm_ptr == NULL) {
        return NULL;
    }
    return ctx;
}

static inline uint32_t solix_vm_read_bytes(
    const SolixVmContext* ctx,
    uint32_t source_address,
    uint8_t* out_buffer,
    uint32_t capacity
) {
    if (ctx == NULL || ctx->read_bytes == NULL || source_address == 0) {
        return 0;
    }
    return ctx->read_bytes(ctx->vm_ptr, source_address, out_buffer, capacity);
}

static inline uint32_t solix_vm_alloc_string_len(
    const SolixVmContext* ctx,
    const char* text,
    uint32_t byte_count
) {
    if (ctx == NULL || ctx->alloc_string == NULL) {
        return 0;
    }
    return ctx->alloc_string(ctx->vm_ptr, (const uint8_t*)text, byte_count);
}

static inline uint32_t solix_vm_alloc_string(const SolixVmContext* ctx, const char* text) {
    if (text == NULL) {
        return solix_vm_alloc_string_len(ctx, "", 0);
    }
    return solix_vm_alloc_string_len(ctx, text, (uint32_t)strlen(text));
}

/**
 * @brief Allocates a NUL-terminated heap buffer holding the guest string at `heap_address`.
 *        Caller owns the returned pointer and must release it with `free()`.
 */
static inline char* solix_vm_read_string_alloc(
    const SolixVmContext* ctx,
    uint32_t heap_address,
    size_t* out_length
) {
    if (ctx == NULL || ctx->read_bytes == NULL || heap_address == 0) {
        if (out_length != NULL) {
            *out_length = 0;
        }
        return NULL;
    }
    uint32_t total_len = ctx->read_bytes(ctx->vm_ptr, heap_address, NULL, 0);
    char* result = (char*)malloc((size_t)total_len + 1U);
    if (result == NULL) {
        if (out_length != NULL) {
            *out_length = 0;
        }
        return NULL;
    }
    if (total_len > 0) {
        ctx->read_bytes(ctx->vm_ptr, heap_address, (uint8_t*)result, total_len);
    }
    result[total_len] = '\0';
    if (out_length != NULL) {
        *out_length = (size_t)total_len;
    }
    return result;
}

static inline uint32_t solix_vm_alloc_chars(
    const SolixVmContext* ctx,
    const char* text,
    uint32_t char_count
) {
    if (ctx == NULL || ctx->alloc_chars == NULL) {
        return 0;
    }
    return ctx->alloc_chars(ctx->vm_ptr, (const uint8_t*)text, char_count);
}

static inline uint32_t solix_vm_alloc_byte_array(
    const SolixVmContext* ctx,
    const uint8_t* raw_bytes,
    uint32_t byte_count
) {
    if (ctx == NULL || ctx->alloc_byte_array == NULL) {
        return 0;
    }
    return ctx->alloc_byte_array(ctx->vm_ptr, raw_bytes, byte_count);
}

static inline uint32_t solix_vm_alloc_array(
    const SolixVmContext* ctx,
    uint32_t element_count,
    uint8_t type_tag
) {
    if (ctx == NULL || ctx->alloc_array == NULL) {
        return 0;
    }
    return ctx->alloc_array(ctx->vm_ptr, element_count, type_tag);
}

static inline uint32_t solix_vm_array_len(const SolixVmContext* ctx, uint32_t array_address) {
    if (ctx == NULL || ctx->array_len == NULL || array_address == 0) {
        return 0;
    }
    return ctx->array_len(ctx->vm_ptr, array_address);
}

static inline uint64_t solix_vm_array_get_u64(
    const SolixVmContext* ctx,
    uint32_t array_address,
    uint32_t element_index
) {
    if (ctx == NULL || ctx->array_get_u64 == NULL || array_address == 0) {
        return 0;
    }
    return ctx->array_get_u64(ctx->vm_ptr, array_address, element_index);
}

static inline void solix_vm_array_set_u64(
    const SolixVmContext* ctx,
    uint32_t array_address,
    uint32_t element_index,
    uint64_t raw_value
) {
    if (ctx != NULL && ctx->array_set_u64 != NULL && array_address != 0) {
        ctx->array_set_u64(ctx->vm_ptr, array_address, element_index, raw_value);
    }
}

static inline uint32_t solix_vm_get_class_vtable_id(
    const SolixVmContext* ctx,
    const char* class_name
) {
    if (ctx == NULL || ctx->get_class_vtable_id == NULL || class_name == NULL) {
        return 0;
    }
    return ctx->get_class_vtable_id(ctx->vm_ptr, class_name);
}

static inline uint32_t solix_vm_alloc_object(
    const SolixVmContext* ctx,
    uint32_t class_vtable_id,
    uint32_t field_count
) {
    if (ctx == NULL || ctx->alloc_object == NULL || class_vtable_id == 0) {
        return 0;
    }
    return ctx->alloc_object(ctx->vm_ptr, class_vtable_id, field_count);
}

static inline int32_t solix_vm_get_field_i32(
    const SolixVmContext* ctx,
    uint32_t object_address,
    uint32_t field_index
) {
    if (ctx == NULL || ctx->get_field_i32 == NULL || object_address == 0) {
        return 0;
    }
    return ctx->get_field_i32(ctx->vm_ptr, object_address, field_index);
}

static inline void solix_vm_set_field_i32(
    const SolixVmContext* ctx,
    uint32_t object_address,
    uint32_t field_index,
    int32_t field_value
) {
    if (ctx != NULL && ctx->set_field_i32 != NULL && object_address != 0) {
        ctx->set_field_i32(ctx->vm_ptr, object_address, field_index, field_value);
    }
}

static inline uint64_t solix_vm_read_field_u64(
    const SolixVmContext* ctx,
    uint32_t object_address,
    uint32_t field_offset
) {
    if (ctx == NULL || ctx->read_field_u64 == NULL || object_address == 0) {
        return 0;
    }
    return ctx->read_field_u64(ctx->vm_ptr, object_address, field_offset);
}

static inline void solix_vm_write_field_u64(
    const SolixVmContext* ctx,
    uint32_t object_address,
    uint32_t field_offset,
    uint64_t raw_value
) {
    if (ctx != NULL && ctx->write_field_u64 != NULL && object_address != 0) {
        ctx->write_field_u64(ctx->vm_ptr, object_address, field_offset, raw_value);
    }
}

static inline uint64_t solix_vm_get_metric(const SolixVmContext* ctx, uint32_t metric_id) {
    if (ctx == NULL || ctx->get_vm_metric == NULL) {
        return 0;
    }
    return ctx->get_vm_metric(ctx->vm_ptr, metric_id);
}

static inline uint64_t solix_vm_collect_cycles(const SolixVmContext* ctx) {
    if (ctx == NULL || ctx->collect_cycles == NULL) {
        return 0;
    }
    return ctx->collect_cycles(ctx->vm_ptr);
}

static inline void solix_vm_set_cycle_threshold(const SolixVmContext* ctx, uint32_t threshold) {
    if (ctx != NULL && ctx->set_cycle_threshold != NULL) {
        ctx->set_cycle_threshold(ctx->vm_ptr, threshold);
    }
}

/**
 * @brief Signals a runtime exception back to the Solix VM by allocating a message string
 *        in the guest heap (falling back to a non-null scalar error tag if no context).
 */
static inline void solix_vm_set_error(
    void* raw_vm_context,
    SolixCValue* out_error,
    const char* message
) {
    if (out_error == NULL) {
        return;
    }
    const SolixVmContext* ctx = solix_vm_context_from_ptr(raw_vm_context);
    if (ctx != NULL && message != NULL) {
        uint32_t str_addr = solix_vm_alloc_string(ctx, message);
        if (str_addr != 0) {
            *out_error = solix_cvalue_ref(str_addr);
            return;
        }
    }
    *out_error = solix_cvalue_bool(false);
}

/* ------------------------------------------------------------------------- */
/* Module Registration Macros                                                */
/* ------------------------------------------------------------------------- */

/**
 * @brief Emits the exported `solix_register_natives` symbol using a static array
 *        of `SolixNativeEntry` descriptors.
 */
#define SOLIX_REGISTER_MODULE(table_array)                                            \
    SOLIX_FFI_EXTERN_C SOLIX_FFI_EXPORT void solix_register_natives(                  \
        void* collector_context,                                                      \
        SolixRegisterCallback register_callback                                       \
    ) {                                                                               \
        if (register_callback == NULL) {                                              \
            return;                                                                   \
        }                                                                             \
        size_t entry_count = sizeof(table_array) / sizeof((table_array)[0]);          \
        for (size_t entry_index = 0; entry_index < entry_count; ++entry_index) {      \
            if ((table_array)[entry_index].symbol != NULL &&                          \
                (table_array)[entry_index].function_pointer != NULL) {                \
                register_callback(                                                    \
                    collector_context,                                                \
                    (table_array)[entry_index].symbol,                                \
                    (table_array)[entry_index].function_pointer                       \
                );                                                                    \
            }                                                                         \
        }                                                                             \
    }

#define SOLIX_EXPORT_NATIVES_BEGIN()                                                  \
    SOLIX_FFI_EXTERN_C SOLIX_FFI_EXPORT void solix_register_natives(                  \
        void* collector_context,                                                      \
        SolixRegisterCallback register_callback                                       \
    ) {                                                                               \
        if (register_callback == NULL) {                                              \
            return;                                                                   \
        }

#define SOLIX_EXPORT_NATIVE(symbol_name, fn_ptr)                                      \
        register_callback(collector_context, (symbol_name), (fn_ptr));

#define SOLIX_EXPORT_NATIVES_END()                                                    \
    }

#ifdef __cplusplus
}
#endif

#endif /* SOLIX_FFI_H */
