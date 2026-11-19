cmake_minimum_required(VERSION 3.20)
project(cmake_backend VERSION 1.0.0 LANGUAGES C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

option(SOLIX_NATIVE_BUILD "Build as a Solix VM native shared library plugin" OFF)
option(BUILD_TESTING "Build standalone unit tests" ON)

if(SOLIX_NATIVE_BUILD)
    include(FetchContent)
    if(DEFINED ENV{SOLIX_FFI_SOURCE_DIR} AND EXISTS "$ENV{SOLIX_FFI_SOURCE_DIR}/CMakeLists.txt")
        file(TO_CMAKE_PATH "$ENV{SOLIX_FFI_SOURCE_DIR}" SOLIX_FFI_LOCAL_DIR)
        FetchContent_Declare(
            solix_ffi
            SOURCE_DIR "${SOLIX_FFI_LOCAL_DIR}"
        )
    else()
        FetchContent_Declare(
            solix_ffi
            GIT_REPOSITORY https://github.com/Jehudme/solix-language.git
            GIT_TAG        dev
            SOURCE_SUBDIR  ffi/c
            GIT_SHALLOW    TRUE
        )
    endif()
    FetchContent_MakeAvailable(solix_ffi)

    add_library(cmake_backend SHARED
        src/matrix_core.c
        src/solix_bridge.cpp
    )
    target_link_libraries(cmake_backend PRIVATE solix::ffi)
    target_compile_definitions(cmake_backend PRIVATE SOLIX_NATIVE_BUILD=1)
else()
    add_library(cmake_backend STATIC
        src/matrix_core.c
    )
    if(BUILD_TESTING)
        enable_testing()
        add_executable(cmake_backend_tests tests/test_cmake_backend.cpp)
        target_link_libraries(cmake_backend_tests PRIVATE cmake_backend)
        add_test(NAME cmake_backend_unit_tests COMMAND cmake_backend_tests)
    endif()
endif()

target_include_directories(cmake_backend PUBLIC
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
)
