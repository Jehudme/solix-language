cmake_minimum_required(VERSION 3.20)
project(solix_ffi VERSION 1.8.0 LANGUAGES C CXX)

add_library(solix_ffi INTERFACE)
add_library(solix::ffi ALIAS solix_ffi)

target_include_directories(
    solix_ffi
    INTERFACE
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
)

target_compile_definitions(
    solix_ffi
    INTERFACE
        SOLIX_FFI_SDK_VERSION="${PROJECT_VERSION}"
)

option(SOLIX_FFI_BUILD_TESTS "Build the solix::ffi C11 and C++17 SDK test suite" OFF)
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME OR SOLIX_FFI_BUILD_TESTS)
    include(CTest)
    if(BUILD_TESTING)
        add_executable(test_solix_ffi_c tests/test_c_ffi.c)
        target_link_libraries(test_solix_ffi_c PRIVATE solix::ffi)
        set_target_properties(test_solix_ffi_c PROPERTIES
            C_STANDARD 11
            C_STANDARD_REQUIRED ON
            C_EXTENSIONS OFF
        )
        if(MSVC)
            target_compile_options(test_solix_ffi_c PRIVATE /W4 /WX)
        else()
            target_compile_options(test_solix_ffi_c PRIVATE -Wall -Wextra -Werror)
        endif()
        add_test(NAME solix_ffi_c11_tests COMMAND test_solix_ffi_c)

        add_executable(test_solix_ffi_cpp tests/test_cpp_ffi.cpp)
        target_link_libraries(test_solix_ffi_cpp PRIVATE solix::ffi)
        set_target_properties(test_solix_ffi_cpp PROPERTIES
            CXX_STANDARD 17
            CXX_STANDARD_REQUIRED ON
            CXX_EXTENSIONS OFF
        )
        if(MSVC)
            target_compile_options(test_solix_ffi_cpp PRIVATE /W4 /WX)
        else()
            target_compile_options(test_solix_ffi_cpp PRIVATE -Wall -Wextra -Werror)
        endif()
        add_test(NAME solix_ffi_cpp17_tests COMMAND test_solix_ffi_cpp)
    endif()
endif()
