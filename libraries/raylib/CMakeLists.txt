cmake_minimum_required(VERSION 3.20)
project(solix_raylib_vendor C)

set(RAYLIB_VERSION_TAG "master" CACHE STRING "Upstream Raylib Git tag")

include(FetchContent)

set(BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(BUILD_GAMES OFF CACHE BOOL "" FORCE)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(CMAKE_POSITION_INDEPENDENT_CODE ON CACHE BOOL "" FORCE)

FetchContent_Declare(
    raylib
    GIT_REPOSITORY https://github.com/raysan5/raylib.git
    GIT_TAG        ${RAYLIB_VERSION_TAG}
    GIT_SHALLOW    TRUE
)
FetchContent_MakeAvailable(raylib)
