cmake_minimum_required(VERSION 3.20)
project(solix_raylib_vendor C)

set(RAYLIB_VERSION "6.2" CACHE STRING "Raylib version tag")

set(FETCHCONTENT_BASE_DIR "${CMAKE_BINARY_DIR}/_deps")

include(FetchContent)

set(BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(BUILD_GAMES OFF CACHE BOOL "" FORCE)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(CMAKE_POSITION_INDEPENDENT_CODE ON CACHE BOOL "" FORCE)

FetchContent_Declare(
    raylib
    GIT_REPOSITORY https://github.com/raysan5/raylib.git
    GIT_TAG        ${RAYLIB_VERSION}
    GIT_SHALLOW    TRUE
)
FetchContent_MakeAvailable(raylib)

# Locate rlparser source file across Raylib versions
if(EXISTS "${raylib_SOURCE_DIR}/tools/rlparser/rlparser.c")
    set(RLPARSER_SOURCE "${raylib_SOURCE_DIR}/tools/rlparser/rlparser.c")
elseif(EXISTS "${raylib_SOURCE_DIR}/parser/raylib_parser.c")
    set(RLPARSER_SOURCE "${raylib_SOURCE_DIR}/parser/raylib_parser.c")
else()
    message(FATAL_ERROR "Could not find rlparser source in Raylib source directory")
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
    DEPENDS rlparser
    WORKING_DIRECTORY "${CMAKE_BINARY_DIR}"
    COMMENT "Extracting C API JSON schema via rlparser: ${API_INPUT_HEADER} -> ${API_OUTPUT_FILE}"
)
