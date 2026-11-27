cmake_minimum_required(VERSION 3.20)
if(POLICY CMP0169)
    cmake_policy(SET CMP0169 OLD)
endif()
project(solix_raylib_vendor C)

set(RAYLIB_VERSION "6.2" CACHE STRING "Raylib release tag")
set(BUILD_HOST_PARSER_ONLY OFF CACHE BOOL "Only build the host rlparser tool and extract API JSON")

set(FETCHCONTENT_BASE_DIR "${CMAKE_BINARY_DIR}/_deps")
include(FetchContent)

get_filename_component(HOST_RAYLIB_SRC "${CMAKE_CURRENT_SOURCE_DIR}/../../build/.cache/cmake/host/_deps/raylib-src" ABSOLUTE)
if(NOT BUILD_HOST_PARSER_ONLY AND EXISTS "${HOST_RAYLIB_SRC}/src/raylib.h")
    FetchContent_Declare(
        raylib
        SOURCE_DIR "${HOST_RAYLIB_SRC}"
    )
else()
    FetchContent_Declare(
        raylib
        GIT_REPOSITORY https://github.com/raysan5/raylib.git
        GIT_TAG        ${RAYLIB_VERSION}
        GIT_SHALLOW    TRUE
    )
endif()

if(BUILD_HOST_PARSER_ONLY)
    FetchContent_GetProperties(raylib)
    if(NOT raylib_POPULATED)
        FetchContent_Populate(raylib)
    endif()

    if(EXISTS "${raylib_SOURCE_DIR}/tools/rlparser/rlparser.c")
        set(RLPARSER_SOURCE "${raylib_SOURCE_DIR}/tools/rlparser/rlparser.c")
    elseif(EXISTS "${raylib_SOURCE_DIR}/parser/raylib_parser.c")
        set(RLPARSER_SOURCE "${raylib_SOURCE_DIR}/parser/raylib_parser.c")
    else()
        message(FATAL_ERROR "Could not find rlparser source in fetched Raylib repository")
    endif()

    add_executable(rlparser ${RLPARSER_SOURCE})

    if(NOT DEFINED API_OUTPUT_FILE)
        set(API_OUTPUT_FILE "${CMAKE_BINARY_DIR}/api/raylib_api.json")
    endif()

    get_filename_component(API_OUTPUT_DIR "${API_OUTPUT_FILE}" DIRECTORY)
    file(MAKE_DIRECTORY "${API_OUTPUT_DIR}")

    if(NOT DEFINED API_INPUT_HEADER)
        set(API_INPUT_HEADER "${raylib_SOURCE_DIR}/src/raylib.h")
    endif()

    if(NOT DEFINED API_DEFINE_GUARD)
        set(API_DEFINE_GUARD "RLAPI")
    endif()

    add_custom_target(extract_api_json
        COMMAND $<TARGET_FILE:rlparser> -i "${API_INPUT_HEADER}" -o "${API_OUTPUT_FILE}" -f JSON -d "${API_DEFINE_GUARD}"
        COMMAND $<TARGET_FILE:rlparser> -i "${raylib_SOURCE_DIR}/src/rlgl.h" -o "${API_OUTPUT_DIR}/rlgl_api.json" -f JSON -d "RLAPI"
        COMMAND $<TARGET_FILE:rlparser> -i "${raylib_SOURCE_DIR}/src/raymath.h" -o "${API_OUTPUT_DIR}/raymath_api.json" -f JSON -d "RMAPI"
        DEPENDS rlparser
        WORKING_DIRECTORY "${CMAKE_BINARY_DIR}"
        COMMENT "Extracting C API JSON schemas (raylib, rlgl, raymath) via rlparser -> ${API_OUTPUT_DIR}"
    )
else()
    set(BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
    set(BUILD_GAMES OFF CACHE BOOL "" FORCE)
    set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
    set(CMAKE_POSITION_INDEPENDENT_CODE ON CACHE BOOL "" FORCE)

    set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib")

    FetchContent_MakeAvailable(raylib)
endif()
