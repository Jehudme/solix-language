#include "solix_ffi.h"

#include <cmath>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <string>
#include <vector>

#define TEST_CHECK(condition)                                                        \
    do {                                                                             \
        if (!(condition)) {                                                          \
            std::cerr << "Test check failed: " << #condition << " (" << __FILE__     \
                      << ":" << __LINE__ << ")\n";                                   \
            std::abort();                                                            \
        }                                                                            \
    } while (0)

namespace {

SolixCValue cpp_multiply(
    void* vm_context,
    const SolixCValue* arguments,
    uint32_t argument_count,
    SolixCValue* out_error
) {
    if (argument_count < 2U || arguments == nullptr) {
        solix_vm_set_error(vm_context, out_error, "expected 2 operands");
        return solix_cvalue_null();
    }
    return solix_cvalue_i64(
        solix_cvalue_as_i64(arguments[0]) * solix_cvalue_as_i64(arguments[1])
    );
}

void cpp_collect_symbol(
    void* collector_context,
    const char* symbol_name,
    SolixNativeFunctionPtr function_pointer
) {
    TEST_CHECK(function_pointer != nullptr);
    auto* registered = static_cast<std::vector<std::string>*>(collector_context);
    registered->emplace_back(symbol_name);
}

}  // namespace

SOLIX_EXPORT_NATIVES_BEGIN()
    SOLIX_EXPORT_NATIVE("CppTestBridge::multiply", cpp_multiply)
SOLIX_EXPORT_NATIVES_END()

int main() {
    TEST_CHECK(solix_cvalue_as_i64(solix_cvalue_i64(-9000000000LL)) == -9000000000LL);
    TEST_CHECK(std::abs(solix_cvalue_as_f64(solix_cvalue_f64(2.7182818)) - 2.7182818) < 1e-12);
    TEST_CHECK(solix_vm_context_from_ptr(nullptr) == nullptr);

    std::vector<std::string> symbols;
    solix_register_natives(&symbols, cpp_collect_symbol);
    TEST_CHECK(symbols.size() == 1U);
    TEST_CHECK(symbols[0] == "CppTestBridge::multiply");

    SolixCValue err = solix_cvalue_null();
    SolixCValue args[2] = { solix_cvalue_i64(6), solix_cvalue_i64(7) };
    SolixCValue prod = cpp_multiply(nullptr, args, 2U, &err);
    TEST_CHECK(solix_cvalue_as_i64(prod) == 42);

    std::cout << "C++17 solix_ffi tests passed.\n";
    return 0;
}
